use core::ptr::addr_of_mut;
use core::sync::atomic::{compiler_fence, Ordering};
use core::{mem::size_of, ptr::addr_of};

use super::pci::{BaseAddressRegister, PCIDevice};
use crate::asm::inb;
use crate::enable_interrupts;
use crate::{
    log,
    memory::{allocate_anywhere, allocate_identity_mapped, EntryFlags, Flags, Page, PhysAddr, Translate, VirtAddr},
    println,
};
use alloc::boxed::Box;
use alloc::vec::Vec;

const AHCI_BASE: VirtAddr = 0x400000;

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
const ATA_SR_BSY: u32 = 0x80;
const ATA_SR_DRQ: u32 = 0x8;

const NO_CACHE: u64 = EntryFlags::NO_CACHE | EntryFlags::WRITABLE;

const ATA_CMD_IDENTIFY: u8 = 0xec;

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
    //// Type of FIS
    fis_type: FISType,
    //// Port multiplier
    pmport: u8,
    //// Command register
    command: u8,
    //// Feature register, 7:0
    featurel: u8,
    //// LBA low register, 7:0
    lba0: u8,
    //// LBA mid register, 15:8
    lba1: u8,
    //// LBA high register, 23:16
    lba2: u8,
    //// Device register
    device: u8,
    //// LBA register, 31:24
    lba3: u8,
    //// LBA register, 39:32
    lba4: u8,
    //// LBA register, 47:40
    lba5: u8,
    //// Feature register, 15:8
    featureh: u8,
    //// Count register, 7:0
    count: u16,
    //// Isochronous command completion
    icc: u8,
    //// Control register
    control: u8,
    //// Reserved
    rsv1: u32,
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
            pmport: 0,
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
            count,
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
            allocate_identity_mapped(abar.base_address(), size_of::<HBA>(), NO_CACHE)
        } else {
            unreachable!()
        };
        Self { pci, hba_ptr }
    }
    pub fn hba(&self) -> &mut HBA {
        unsafe { &mut *(self.hba_ptr as *mut HBA) }
    }
    pub fn initialize(&mut self) {
        // if !self.reset() {
        //     log!("AHCI controller reset failed!");
        //     return
        // }
        // Ensure that HBA knows we are AHCI aware.
        self.hba().control_regs.ghc = 0x80000000;
        // Enable interrupt line
        log!("Enabling PCI interrupt pin {} line {}", self.pci.interrupt_pin.read(), self.pci.interrupt_line.read());
        self.pci.common.command.write(self.pci.common.command.read() & !(1 << 10));
        // Enable bus mastering
        self.pci.common.command.write(self.pci.common.command.read() | (1 << 0) | (1 << 2));
        // Enable global interrupts
        self.hba().control_regs.ghc |= 1 << 1;
    }
    fn reset(&mut self) -> bool {
        self.hba().control_regs.ghc = 1;
        // dbgln_if(AHCI_DEBUG, "{}: AHCI Controller reset", pci_address());
        full_memory_barrier();
        let mut retry = 0;
        loop {
            if retry > 1000 {
                return false;
            }
            if (self.hba().control_regs.ghc & 1) == 0 {
                break;
            }
            delay(1000);
            retry += 1;
        }
        // The HBA is locked or hung if we waited more than 1 second!
        return true;
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
                        let reg_ptr = addr_of!(self.hba().port_regs[idx]) as VirtAddr;
                        let mut port = AHCIPort::new(idx, self, reg_ptr);
                        port.initialize();
                        ports.push(Box::new(port))
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
                        log!("    Garbage device signature {:#x} on port {} - ignored", signature, idx);
                    }
                }
            }
        }
        for port in &ports {
            self.rebase(port);
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
    fn start_port(&self, idx: usize) {
        let ptr = addr_of_mut!(self.hba().port_regs[idx].cmd);
        unsafe {
            while (ptr.read_volatile() & HBA_PXCMD_CR) > 0 {}
            ptr.write_volatile(ptr.read_volatile() | HBA_PXCMD_FRE);
            ptr.write_volatile(ptr.read_volatile() | HBA_PXCMD_ST);
        }
    }
    fn rebase(&self, port: &AHCIPort) {
        self.stop_port(port.number);
        unsafe {
            let page_addr = allocate_anywhere(command_list_base(port.number), 1024, NO_CACHE);
            addr_of_mut!(port.registers().clb).write_volatile(page_addr.translate() as u32);
            addr_of_mut!(port.registers().clbu).write_volatile(0);
            let page_addr = allocate_anywhere(fis_base(port.number), 256, NO_CACHE);
            addr_of_mut!(port.registers().fb).write_volatile(page_addr.translate() as u32);
            addr_of_mut!(port.registers().fbu).write_volatile(0);
            let hba_cmd_hdr = command_list_base(port.number) as *mut CommandHeader;
            for cmd_idx in 0..32 {
                let ptr = addr_of_mut!((*(hba_cmd_hdr.offset(cmd_idx))).prdtl);
                ptr.write_volatile(8);
                let ctba = command_table_descriptor(port.number, cmd_idx as usize);
                let page_addr = allocate_anywhere(ctba, 256, NO_CACHE);
                let ptr = addr_of_mut!((*(hba_cmd_hdr.offset(cmd_idx))).ctba);
                ptr.write_volatile(page_addr.translate() as u32);
                let ptr = addr_of_mut!((*(hba_cmd_hdr.offset(cmd_idx))).ctbau);
                ptr.write_volatile(0);
            }
        }
        self.start_port(port.number);
    }
}

fn fis_base(port_number: usize) -> VirtAddr {
    AHCI_BASE + (32 << 10) + (port_number << 8)
}

fn command_list_base(port_number: usize) -> VirtAddr {
    AHCI_BASE + (port_number << 10)
}

//// Command table descriptor base address
fn command_table_descriptor(port_number: usize, cmd_idx: usize) -> VirtAddr {
    AHCI_BASE + (40 << 10) + (port_number << 13) + ((cmd_idx as usize) << 8)
}

#[derive(Debug)]
#[repr(C, packed)]
struct PortRegisters {
    //// Port x Command List Base Address
    clb: u32,
    //// Port x Command List Base Address Upper 32-Bits
    clbu: u32,
    //// Port x FIS Base Address
    fb: u32,
    //// Port x FIS Base Address Upper 32-Bits
    fbu: u32,
    //// Port x Interrupt Status
    is: u32,
    //// Port x Interrupt Enable
    ie: u32,
    //// Port x Command and Status
    cmd: u32,
    reserved: u32,
    //// Port x Task File Data
    tfd: u32,
    //// Port x Signature
    sig: u32,
    //// Port x Serial ATA Status (SCR0: SStatus)
    ssts: u32,
    //// Port x Serial ATA Control (SCR2: SControl)
    sctl: u32,
    //// Port x Serial ATA Error (SCR1: SError)
    serr: u32,
    //// Port x Serial ATA Active (SCR3: SActive)
    sact: u32,
    //// Port x Command Issue
    ci: u32,
    //// Port x Serial ATA Notification (SCR4: SNotification)
    sntf: u32,
    //// Port x FIS-based Switching Control
    fbs: u32,
    //// Port x Device Sleep
    devslp: u32,
    reserved2: [u8; 0x70 - 0x48],
    //// Port x Vendor Specific
    vs: [u8; 16],
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct GenericHostControl {
    //// Host Capabilities
    cap: u32,
    //// Global Host Control
    ghc: u32,
    //// Interrupt Status
    is: u32,
    //// Ports Implemented
    pi: u32,
    version: u32,
    //// Command Completion Coalescing Control
    ccc_ctl: u32,
    //// Command Completion Coalsecing Ports
    ccc_ports: u32,
    //// Enclosure Management Location
    em_loc: u32,
    //// Enclosure Management Control
    em_ctl: u32,
    //// Host Capabilities Extended
    cap2: u32,
    //// BIOS/OS Handoff Control and Status
    bohc: u32,
}

#[derive(Debug)]
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
    //// Physical Region Descriptor Table Length
    prdtl: u16,
    //// Physical Region Descriptor Byte Count
    prdbc: u32,
    //// Command Table Descriptor Base Address
    ctba: u32,
    //// Command Table Descriptor Base Address Upper 32-bits
    ctbau: u32,
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

pub struct AHCIPort<'a> {
    number: usize,
    parent: &'a AHCIController,
    reg_ptr: VirtAddr,
    metadata_addr: PhysAddr,
    wait_for_completion: bool,
}

impl<'a> AHCIPort<'a> {
    fn new(number: usize, parent: &'a AHCIController, reg_ptr: VirtAddr) -> Self {
        let metadata_addr = allocate_identity_mapped(2 * AHCI_BASE + number << 10, 512, NO_CACHE).translate();
        Self { number, parent, reg_ptr, metadata_addr, wait_for_completion: true }
    }
    fn registers(&self) -> &'a mut PortRegisters {
        unsafe { &mut *(self.reg_ptr as *mut PortRegisters) }
    }
    //// Find a free command list slot
    fn find_cmd_slot(&self) -> Option<usize> {
        // If not set in SACT and CI, the slot is free
        let mut slots = self.registers().sact | self.registers().ci;
        for i in 0..32 {
            if slots & 1 == 0 {
                return Some(i);
            }
            slots >>= 1;
        }
        // trace_ahci("Cannot find free command list entry\n");
        None
    }
    pub fn initialize(&mut self) {
        self.power_on();
        self.spin_up();
        self.clear_sata_error_register();
        self.start_fis_receiving();
        self.set_active_state();
        // self.registers().sctl = (self.registers().sctl & 0xfffffff0) | 1;
        // delay(1000);
        self.registers().is = 0xffffffff;
        self.registers().ie = 0xffffffff;
        full_memory_barrier();
        // This actually enables the port...
        self.start_command_list_processing();
        full_memory_barrier();
    }
    fn power_on(&mut self) {
        if (self.registers().cmd & (1 << 20)) == 0 {
            return;
        }
        self.registers().cmd = self.registers().cmd | (1 << 2)
    }
    fn spin_up(&mut self) {
        if (self.parent.hba().control_regs.cap & (1 << 27)) == 0 {
            // No staggered spin up support
            return;
        }
        self.registers().cmd = self.registers().cmd | (1 << 1)
    }
    fn clear_sata_error_register(&mut self) {
        self.registers().serr = self.registers().serr;
    }
    fn start_fis_receiving(&mut self) {
        let ptr = addr_of_mut!((*self.registers()).cmd);
        unsafe { ptr.write_volatile(ptr.read() | (1 << 4)) };
    }
    fn set_active_state(&mut self) {
        let ptr = addr_of_mut!((*self.registers()).cmd);
        unsafe { ptr.write_volatile(ptr.read() & 0x0ffffff | (1 << 28)) };
    }
    fn start_command_list_processing(&mut self) {
        let ptr = addr_of_mut!((*self.registers()).cmd);
        unsafe { ptr.write_volatile(ptr.read() | 1) };
    }
    pub fn identify(&mut self) {
        let slot = self.find_cmd_slot().unwrap() as isize;
        self.wait_for_completion = true;
        unsafe {
            let cmd_hdr = command_list_base(self.number);
            let cmd_hdr = (cmd_hdr as *mut CommandHeader).offset(slot);
            let ptr = addr_of_mut!((*cmd_hdr).prdtl);
            ptr.write_volatile(1);
            let ptr = addr_of_mut!((*cmd_hdr).prdbc);
            ptr.write_volatile(256);
            let cmd_table = command_table_descriptor(self.number, slot as usize) as *mut CommandTable;
            let ptr = addr_of_mut!((*cmd_table).descriptors[0]);
            ptr.write_volatile(PhysicalRegionDescriptor {
                base_low: self.metadata_addr as u32,
                base_high: 0,
                reserved: 0,
                byte_count: 511,
            });
            let ptr = addr_of_mut!((*cmd_hdr).attributes);
            ptr.write_volatile(5 | 128);
            let fis = addr_of_mut!((*cmd_table).command_fis) as *mut RegisterH2D;
            let ptr = addr_of_mut!((*fis).fis_type);
            ptr.write_volatile(FISType::RegisterH2D);
            let ptr = addr_of_mut!((*fis).pmport);
            ptr.write_volatile(128);
            let ptr = addr_of_mut!((*fis).command);
            ptr.write_volatile(ATA_CMD_IDENTIFY);
            self.spin_until_ready();
            let ptr = addr_of_mut!((*self.registers()).ci);
            ptr.write_volatile(ptr.read() | (1 << slot));
            log!("{:x?}", self.registers());
            log!("{:x?}", self.parent.hba());
            loop {
                if self.registers().serr != 0 {
                    panic!("Error accessing AHCI device")
                }
                if !self.wait_for_completion {
                    break;
                }
            }
        }
    }
    fn spin_until_ready(&self) {
        let mut spin = 0;
        while (self.registers().tfd & (ATA_SR_BSY | ATA_SR_DRQ) > 0) && spin <= 100 {
            delay(1000);
            spin += 1;
        }
        if spin == 100 {
            panic!("Device is not responding!")
        }
    }
}

fn delay(microseconds: usize) {
    for _ in 0..microseconds {
        unsafe {
            inb(0x80);
        }
    }
}

fn full_memory_barrier() {
    compiler_fence(Ordering::AcqRel);
}
