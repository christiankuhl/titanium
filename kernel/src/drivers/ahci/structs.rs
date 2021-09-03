use crate::memory::{EntryFlags, Flags};

pub const SATA_DRIVE: u32 = 0x00000101; // SATA drive
pub const ATAPI_DRIVE: u32 = 0xEB140101; // SATAPI drive
pub const EM_BRIDGE: u32 = 0xC33C0101; // Enclosure management bridge
pub const PORT_MULTIPLIER: u32 = 0x96690101; // Port multiplier
pub const HBA_PORT_IPM_ACTIVE: u32 = 1;
pub const HBA_PORT_DET_PRESENT: u32 = 3;
pub const HBA_PXCMD_ST: u32 = 0x0001;
pub const HBA_PXCMD_FRE: u32 = 0x0010;
pub const HBA_PXCMD_FR: u32 = 0x4000;
pub const HBA_PXCMD_CR: u32 = 0x8000;
pub const ATA_SR_BSY: u32 = 0x80;
pub const ATA_SR_DRQ: u32 = 0x8;
pub const NO_CACHE: u64 = EntryFlags::NO_CACHE | EntryFlags::WRITABLE;
pub const ATA_CMD_IDENTIFY: u8 = 0xec;

//// Type of FIS (= Frame Information Structure)
#[repr(u8)]
pub enum FISType {
    //// Register FIS - host to device
    RegisterH2D = 0x27,
    //// Register FIS - device to host
    RegisterD2H = 0x34,
    //// DMA activate FIS - device to host
    DMAActivate = 0x39,
    //// DMA setup FIS - bidirectional
    DMASetup = 0x41,
    //// Data FIS - bidirectional
    Data = 0x46,
    //// BIST activate FIS - bidirectional
    BISTActivate = 0x58,
    //// PIO setup FIS - device to host
    PIOSetup = 0x5F,
    //// Set device bits FIS - device to host
    SetDeviceBits = 0xA1,
}

#[repr(u8)]
pub enum CommandControl {
    Control = 0,
    Command = 1,
}

#[repr(C, packed)]
pub struct RegisterH2D {
    //// Type of FIS
    pub fis_type: FISType,
    //// Port multiplier
    pub pmport: u8,
    //// Command register
    pub command: u8,
    //// Feature register, 7:0
    pub featurel: u8,
    //// LBA low register, 7:0
    pub lba0: u8,
    //// LBA mid register, 15:8
    pub lba1: u8,
    //// LBA high register, 23:16
    pub lba2: u8,
    //// Device register
    pub device: u8,
    //// LBA register, 31:24
    pub lba3: u8,
    //// LBA register, 39:32
    pub lba4: u8,
    //// LBA register, 47:40
    pub lba5: u8,
    //// Feature register, 15:8
    pub featureh: u8,
    //// Count register, 7:0
    pub count: u16,
    //// Isochronous command completion
    pub icc: u8,
    //// Control register
    pub control: u8,
    //// Reserved
    rsv1: u32,
}

#[repr(C, packed)]
pub struct PortRegisters {
    //// Port x Command List Base Address
    pub clb: u32,
    //// Port x Command List Base Address Upper 32-Bits
    pub clbu: u32,
    //// Port x FIS Base Address
    pub fb: u32,
    //// Port x FIS Base Address Upper 32-Bits
    pub fbu: u32,
    //// Port x Interrupt Status
    pub is: u32,
    //// Port x Interrupt Enable
    pub ie: u32,
    //// Port x Command and Status
    pub cmd: u32,
    reserved: u32,
    //// Port x Task File Data
    pub tfd: u32,
    //// Port x Signature
    pub sig: u32,
    //// Port x Serial ATA Status (SCR0: SStatus)
    pub ssts: u32,
    //// Port x Serial ATA Control (SCR2: SControl)
    pub sctl: u32,
    //// Port x Serial ATA Error (SCR1: SError)
    pub serr: u32,
    //// Port x Serial ATA Active (SCR3: SActive)
    pub sact: u32,
    //// Port x Command Issue
    pub ci: u32,
    //// Port x Serial ATA Notification (SCR4: SNotification)
    pub sntf: u32,
    //// Port x FIS-based Switching Control
    pub fbs: u32,
    //// Port x Device Sleep
    pub devslp: u32,
    reserved2: [u8; 0x70 - 0x48],
    //// Port x Vendor Specific
    pub vs: [u8; 16],
}

#[repr(C, packed)]
pub struct GenericHostControl {
    //// Host Capabilities
    pub cap: u32,
    //// Global Host Control
    pub ghc: u32,
    //// Interrupt Status
    pub is: u32,
    //// Ports Implemented
    pub pi: u32,
    pub version: u32,
    //// Command Completion Coalescing Control
    pub ccc_ctl: u32,
    //// Command Completion Coalsecing Ports
    pub ccc_ports: u32,
    //// Enclosure Management Location
    pub em_loc: u32,
    //// Enclosure Management Control
    pub em_ctl: u32,
    //// Host Capabilities Extended
    pub cap2: u32,
    //// BIOS/OS Handoff Control and Status
    pub bohc: u32,
}

#[repr(C, packed)]
pub struct HBA {
    pub control_regs: GenericHostControl,
    reserved: [u8; 52],
    pub nvmhci: [u8; 64],
    pub vendor_specific: [u8; 96],
    pub port_regs: [PortRegisters; 32],
}

#[repr(C, packed)]
pub struct CommandHeader {
    pub attributes: u16,
    //// Physical Region Descriptor Table Length
    pub prdtl: u16,
    //// Physical Region Descriptor Byte Count
    pub prdbc: u32,
    //// Command Table Descriptor Base Address
    pub ctba: u32,
    //// Command Table Descriptor Base Address Upper 32-bits
    pub ctbau: u32,
    reserved: [u32; 4],
}

#[repr(C, packed)]
pub struct PhysicalRegionDescriptor {
    pub base_low: u32,
    pub base_high: u32,
    pub reserved: u32,
    pub byte_count: u32, /* Bit 31 - Interrupt completion, Bit 0 to 21 - Data Byte Count */
}

#[repr(C, packed)]
pub struct CommandTable {
    pub command_fis: [u8; 64],
    pub atapi_command: [u8; 32],
    reserved: [u8; 32],
    pub descriptors: [PhysicalRegionDescriptor; 5], //FIXME!
}
