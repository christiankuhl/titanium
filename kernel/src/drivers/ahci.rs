use core::mem::size_of;
use core::ptr::addr_of_mut;

use super::pci::{BaseAddressRegister, PCIDevice};
use crate::{
    log,
    memory::{allocate_kernel_region, EntryFlags, Flags, VirtAddr},
    println,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use volatile::Volatile;

const SATA_DRIVE: u32 = 0x00000101; // SATA drive
const ATAPI_DRIVE: u32 = 0xEB140101; // SATAPI drive
const EM_BRIDGE: u32 = 0xC33C0101; // Enclosure management bridge
const PORT_MULTIPLIER: u32 = 0x96690101; // Port multiplier
const HBA_PORT_IPM_ACTIVE: u32 = 1;
const HBA_PORT_DET_PRESENT: u32 = 3;
const HBA_PXCMD_ST: u32 = 0x0001;
const HBA_PXCMD_FRE: u32 = 0x0010;
const HBA_PXCMD_FR: u32 = 0x4000;
const HBA_PXCMD_CR: u32 = 0x8000;

#[repr(u8)]
enum FISType {
    // Type of FIS (= Frame Information Structure)
    RegisterH2D = 0x27,   // Register FIS - host to device
    RegisterD2H = 0x34,   // Register FIS - device to host
    DMAActivate = 0x39,   // DMA activate FIS - device to host
    DMASetup = 0x41,      // DMA setup FIS - bidirectional
    Data = 0x46,          // Data FIS - bidirectional
    BISTActivate = 0x58,  // BIST activate FIS - bidirectional
    PIOSetup = 0x5F,      // PIO setup FIS - device to host
    SetDeviceBits = 0xA1, // Set device bits FIS - device to host
}

#[repr(u8)]
enum CommandControl {
    Control = 0,
    Command = 1,
}

#[repr(C, packed)]
struct RegisterH2D {
    fis_type: FISType, // FIS_TYPE_REG_H2D
    pmport: u8,        // Port multiplier
    rsv0: u8,          // Reserved
    c: CommandControl, // 1: Command, 0: Control
    command: u8,       // Command register
    featurel: u8,      // Feature register, 7:0
    lba0: u8,          // LBA low register, 7:0
    lba1: u8,          // LBA mid register, 15:8
    lba2: u8,          // LBA high register, 23:16
    device: u8,        // Device register
    lba3: u8,          // LBA register, 31:24
    lba4: u8,          // LBA register, 39:32
    lba5: u8,          // LBA register, 47:40
    featureh: u8,      // Feature register, 15:8
    countl: u8,        // Count register, 7:0
    counth: u8,        // Count register, 15:8
    icc: u8,           // Isochronous command completion
    control: u8,       // Control register
    rsv1: u64,         // Reserved
}

impl RegisterH2D {
    pub fn new(
        cmdctrl: CommandControl,
        cmdreg: u8,
        ctrlreg: u8,
        device: u8,
        feature: u16,
        lba: u64,
        count: u16,
        icc: u8,
    ) -> Self {
        Self {
            fis_type: FISType::RegisterH2D,
            pmport: 4,
            rsv0: 3,
            c: cmdctrl,
            command: cmdreg,
            featurel: feature as u8,
            lba0: lba as u8,
            lba1: (lba >> 8) as u8,
            lba2: (lba >> 16) as u8,
            device,
            lba3: (lba >> 24) as u8,
            lba4: (lba >> 32) as u8,
            lba5: (lba >> 40) as u8,
            featureh: (feature >> 8) as u8,
            countl: count as u8,
            counth: (count >> 8) as u8,
            icc,
            control: ctrlreg,
            rsv1: 0,
        }
    }
}

pub struct AHCIController {
    pci: Box<PCIDevice>,
    hba_ptr: VirtAddr,
}

impl AHCIController {
    pub fn new(pci: Box<PCIDevice>) -> Self {
        let hba_ptr = if let BaseAddressRegister::MemoryMapped(abar) = pci.bar[5] {
            allocate_kernel_region(abar.base_address(), size_of::<HBA>(), EntryFlags::NO_CACHE | EntryFlags::WRITABLE)
        } else {
            unreachable!()
        };
        Self { pci, hba_ptr }
    }
    pub fn hba(&self) -> &mut HBA {
        unsafe { &mut *(self.hba_ptr as *mut HBA) }
    }
    pub fn enumerate_ports(&self) -> Vec<Box<AHCIPort>> {
        let mut ports = Vec::new();
        let pi = self.hba().control_regs.pi;
        for idx in 0..32 {
            let ssts = self.hba().port_regs[idx].ssts;
            let ipm = (ssts >> 8) & 0xf == HBA_PORT_IPM_ACTIVE;
            let det = ssts & 0xf == HBA_PORT_DET_PRESENT;
            if pi & (1 << idx) > 0 && ipm && det {
                let signature = self.hba().port_regs[idx].sig;
                match signature {
                    SATA_DRIVE => {
                        log!("    SATA drive detected on port {}", idx);
                        ports.push(Box::new(AHCIPort::new(self)))
                    }
                    ATAPI_DRIVE => {
                        log!("    ATAPI drive detected on port {}", idx);
                    }
                    EM_BRIDGE => {
                        log!("    Enclosure management bridge detected on port {}", idx);
                    }
                    PORT_MULTIPLIER => {
                        log!("    Port multiplier detected on port {}", idx);
                    }
                    _ => {
                        log!("    Garbage device signature on port {} - ignored", idx);
                    }
                }
            }
        }
        if ports.len() > 0 {
            println!("Interrupt pin {} line {}", self.pci.interrupt_pin.read(), self.pci.interrupt_line.read());
            self.stop_port(0)
        }
        ports
    }
    fn stop_port(&self, idx: usize) {
        let ptr = addr_of_mut!(self.hba().port_regs[idx].cmd);
        unsafe {
            ptr.write_volatile(ptr.read_volatile() & !HBA_PXCMD_ST);
            ptr.write_volatile(ptr.read_volatile() & !HBA_PXCMD_FRE);
            loop {
                if (ptr.read_volatile() & HBA_PXCMD_FR) > 0 {
                    continue;
                }
                if (ptr.read_volatile() & HBA_PXCMD_CR) > 0 {
                    continue;
                }
                break;
            }
        }
    }
}

#[repr(C, packed)]
struct PortRegisters {
    clb: u32,  /* Port x Command List Base Address */
    clbu: u32, /* Port x Command List Base Address Upper 32-Bits */
    fb: u32,   /* Port x FIS Base Address */
    fbu: u32,  /* Port x FIS Base Address Upper 32-Bits */
    is: u32,   /* Port x Interrupt Status */
    ie: u32,   /* Port x Interrupt Enable */
    cmd: u32,  /* Port x Command and Status */
    reserved: u32,
    tfd: u32,    /* Port x Task File Data */
    sig: u32,    /* Port x Signature */
    ssts: u32,   /* Port x Serial ATA Status (SCR0: SStatus) */
    sctl: u32,   /* Port x Serial ATA Control (SCR2: SControl) */
    serr: u32,   /* Port x Serial ATA Error (SCR1: SError) */
    sact: u32,   /* Port x Serial ATA Active (SCR3: SActive) */
    ci: u32,     /* Port x Command Issue */
    sntf: u32,   /* Port x Serial ATA Notification (SCR4: SNotification) */
    fbs: u32,    /* Port x FIS-based Switching Control */
    devslp: u32, /* Port x Device Sleep */
    reserved2: [u8; 0x70 - 0x48],
    vs: [u8; 16], /* Port x Vendor Specific */
}

#[repr(C, packed)]
pub struct GenericHostControl {
    cap: u32, /* Host Capabilities */
    ghc: u32, /* Global Host Control */
    is: u32,  /* Interrupt Status */
    pi: u32,  /* Ports Implemented */
    version: u32,
    ccc_ctl: u32,   /* Command Completion Coalescing Control */
    ccc_ports: u32, /* Command Completion Coalsecing Ports */
    em_loc: u32,    /* Enclosure Management Location */
    em_ctl: u32,    /* Enclosure Management Control */
    cap2: u32,      /* Host Capabilities Extended */
    bohc: u32,      /* BIOS/OS Handoff Control and Status */
}

#[repr(C, packed)]
pub struct HBA {
    control_regs: GenericHostControl,
    reserved: [u8; 52],
    nvmhci: [u8; 64],
    vendor_specific: [u8; 96],
    port_regs: [PortRegisters; 32],
}

#[repr(C, packed)]
pub struct CommandHeader {
    attributes: u16,
    prdtl: u16, /* Physical Region Descriptor Table Length */
    prdbc: u32, /* Physical Region Descriptor Byte Count */
    ctba: u32,  /* Command Table Descriptor Base Address */
    ctbau: u32, /* Command Table Descriptor Base Address Upper 32-bits */
    reserved: [u32; 4],
}

#[repr(C, packed)]
pub struct PhysicalRegionDescriptor {
    base_low: u32,
    base_high: u32,
    reserved: u32,
    byte_count: u32, /* Bit 31 - Interrupt completion, Bit 0 to 21 - Data Byte Count */
}

#[repr(C, packed)]
pub struct CommandTable {
    command_fis: [u8; 64],
    atapi_command: [u8; 32],
    reserved: [u8; 32],
    descriptors: [PhysicalRegionDescriptor; 5], //FIXME!
}

// typedef struct tagFIS_REG_D2H
// {
// 	// DWORD 0
// 	uint8_t  fis_type;    // FIS_TYPE_REG_D2H

// 	uint8_t  pmport:4;    // Port multiplier
// 	uint8_t  rsv0:2;      // Reserved
// 	uint8_t  i:1;         // Interrupt bit
// 	uint8_t  rsv1:1;      // Reserved

// 	uint8_t  status;      // Status register
// 	uint8_t  error;       // Error register

// 	// DWORD 1
// 	uint8_t  lba0;        // LBA low register, 7:0
// 	uint8_t  lba1;        // LBA mid register, 15:8
// 	uint8_t  lba2;        // LBA high register, 23:16
// 	uint8_t  device;      // Device register

// 	// DWORD 2
// 	uint8_t  lba3;        // LBA register, 31:24
// 	uint8_t  lba4;        // LBA register, 39:32
// 	uint8_t  lba5;        // LBA register, 47:40
// 	uint8_t  rsv2;        // Reserved

// 	// DWORD 3
// 	uint8_t  countl;      // Count register, 7:0
// 	uint8_t  counth;      // Count register, 15:8
// 	uint8_t  rsv3[2];     // Reserved

// 	// DWORD 4
// 	uint8_t  rsv4[4];     // Reserved
// } FIS_REG_D2H;

pub struct AHCIPort<'a> {
    parent: &'a AHCIController,
}

impl<'a> AHCIPort<'a> {
    fn new(parent: &'a AHCIController) -> Self {
        Self { parent }
    }
}
