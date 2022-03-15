use core::fmt::Display;

use super::{BaseAddressRegisters, Register, BDF};

#[derive(Copy, Clone, Debug)]
pub enum PCIDevice {
    Standard(StandardPCIDevice),
    PCIBridge(PCIBridge),
    PCICardBus(PCICardBusBridge),
}

impl PCIDevice {
    pub fn new(bdf: BDF) -> Option<Self> {
        let header_type: u32 = Register::<u32>::new(bdf, 0xc).read().into();
        let multifunction = header_type & 0x800000 > 0;
        let common = CommonPCIHeader::new(bdf, multifunction);
        match (header_type & 0x7f0000) >> 16 {
            0 => Some(Self::Standard(StandardPCIDevice::new(bdf, common))),
            1 => Some(Self::PCIBridge(PCIBridge::new(bdf, common))),
            2 => Some(Self::PCICardBus(PCICardBusBridge::new(bdf, common))),
            _ => None,
        }
    }
    pub fn common(&self) -> CommonPCIHeader {
        match self {
            Self::Standard(device) => device.common,
            Self::PCIBridge(device) => device.common,
            Self::PCICardBus(device) => device.common,
        }
    }
    pub fn bdf(&self) -> BDF {
        match self {
            Self::Standard(device) => device.bdf,
            Self::PCIBridge(device) => device.bdf,
            Self::PCICardBus(device) => device.bdf,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CommonPCIHeader {
    pub device_id: Register<u16>,
    pub vendor_id: Register<u16>,
    pub status: Register<u16>,
    pub command: Register<u16>,
    pub class_id: Register<u8>,
    pub subclass_id: Register<u8>,
    pub interface_id: Register<u8>,
    pub revision: Register<u8>,
    pub bist: Register<u8>,
    pub multifunction: bool,
    pub latency_timer: Register<u8>,
    pub cache_line_size: Register<u8>,
}

impl CommonPCIHeader {
    fn new(bdf: BDF, multifunction: bool) -> Self {
        Self {
            device_id: Register::new(bdf, 0x0),
            vendor_id: Register::new(bdf, 0x2),
            status: Register::new(bdf, 0x4),
            command: Register::new(bdf, 0x6),
            class_id: Register::new(bdf, 0x8),
            subclass_id: Register::new(bdf, 0x9),
            interface_id: Register::new(bdf, 0xa),
            revision: Register::new(bdf, 0xb),
            bist: Register::new(bdf, 0xc),
            multifunction,
            latency_timer: Register::new(bdf, 0xe),
            cache_line_size: Register::new(bdf, 0xf),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StandardPCIDevice {
    pub bdf: BDF,
    pub common: CommonPCIHeader,
    pub bar: BaseAddressRegisters,
    pub cis_pointer: Register<u32>,
    pub subsystem_id: Register<u16>,
    pub subsystem_vendor_id: Register<u16>,
    pub expansion_base: Register<u32>,
    pub cap_pointer: Register<u8>,
    pub max_latency: Register<u8>,
    pub min_grant: Register<u8>,
    pub interrupt_pin: Register<u8>,
    pub interrupt_line: Register<u8>,
}

impl StandardPCIDevice {
    fn new(bdf: BDF, common: CommonPCIHeader) -> Self {
        Self {
            bdf,
            common,
            bar: BaseAddressRegisters::new(),
            cis_pointer: Register::new(bdf, 0x28),
            subsystem_id: Register::new(bdf, 0x2c),
            subsystem_vendor_id: Register::new(bdf, 0x2e),
            expansion_base: Register::new(bdf, 0x30),
            cap_pointer: Register::new(bdf, 0x37),
            max_latency: Register::new(bdf, 0x3c),
            min_grant: Register::new(bdf, 0x3d),
            interrupt_pin: Register::new(bdf, 0x3e),
            interrupt_line: Register::new(bdf, 0x3f),
        }
    }
    pub fn configure(&mut self) {
        self.bar.configure(&PCIDevice::Standard(*self));
    }
    pub fn interrupt_vector(&self) -> u8 {
        self.interrupt_line.read() as u8
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PCIBridge {
    pub bdf: BDF,
    pub common: CommonPCIHeader,
    pub bar: BaseAddressRegisters,
    pub sec_latency_timer: Register<u8>,
    pub subord_bus: Register<u8>,
    pub secondary_bus: Register<u8>,
    pub primary_bus: Register<u8>,
    pub secondary_status: Register<u16>,
    pub io_limit: u32,
    pub io_base: u32,
    pub memory_limit: Register<u16>,
    pub memory_base: Register<u16>,
    pub prefetchable_limit: usize,
    pub prefetchable_base: usize,
    pub cap_pointer: Register<u8>,
    pub expansion_base: Register<u32>,
    pub bridge_ctrl: Register<u16>,
    pub interrupt_pin: Register<u8>,
    pub interrupt_line: Register<u8>,
}

impl PCIBridge {
    fn new(bdf: BDF, common: CommonPCIHeader) -> Self {
        Self {
            bdf,
            common,
            bar: BaseAddressRegisters::new(),
            sec_latency_timer: Register::new(bdf, 0x18),
            subord_bus: Register::new(bdf, 0x19),
            secondary_bus: Register::new(bdf, 0x1a),
            primary_bus: Register::new(bdf, 0x1b),
            secondary_status: Register::new(bdf, 0x1c),
            io_limit: 0,
            io_base: 0,
            memory_limit: Register::new(bdf, 0x20),
            memory_base: Register::new(bdf, 0x22),
            prefetchable_limit: 0,
            prefetchable_base: 0,
            cap_pointer: Register::new(bdf, 0x37),
            expansion_base: Register::new(bdf, 0x38),
            bridge_ctrl: Register::new(bdf, 0x3c),
            interrupt_pin: Register::new(bdf, 0x3e),
            interrupt_line: Register::new(bdf, 0x3f),
        }
    }
    fn configure(&mut self) {
        self.bar.configure(&PCIDevice::PCIBridge(*self));
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PCICardBusBridge {
    pub bdf: BDF,
    pub common: CommonPCIHeader,
    pub cardbus_base: Register<u32>,
    pub secondary_status: Register<u16>,
    pub cap_offset: Register<u8>,
    pub cardbus_latency_timer: Register<u8>,
    pub subordinate_bus: Register<u8>,
    pub cardbus: Register<u8>,
    pub pci_bus: Register<u8>,
    pub memory_base: [u32; 2],
    pub memory_limit: [u32; 2],
    pub io_base: [u32; 2],
    pub io_limit: [u32; 2],
    pub bridge_ctrl: Register<u16>,
    pub interrupt_pin: Register<u8>,
    pub interrupt_line: Register<u8>,
    pub subsystem_vendor_id: Register<u16>,
    pub subsystem_device_id: Register<u16>,
    pub legacy_base: Register<u32>,
}

impl PCICardBusBridge {
    fn new(bdf: BDF, common: CommonPCIHeader) -> Self {
        Self {
            bdf,
            common,
            cardbus_base: Register::new(bdf, 0x10),
            secondary_status: Register::new(bdf, 0x14),
            cap_offset: Register::new(bdf, 0x17),
            cardbus_latency_timer: Register::new(bdf, 0x18),
            subordinate_bus: Register::new(bdf, 0x19),
            cardbus: Register::new(bdf, 0x1a),
            pci_bus: Register::new(bdf, 0x1b),
            memory_base: [0; 2],
            memory_limit: [0; 2],
            io_base: [0; 2],
            io_limit: [0; 2],
            bridge_ctrl: Register::new(bdf, 0x3c),
            interrupt_pin: Register::new(bdf, 0x3e),
            interrupt_line: Register::new(bdf, 0x3f),
            subsystem_vendor_id: Register::new(bdf, 0x40),
            subsystem_device_id: Register::new(bdf, 0x42),
            legacy_base: Register::new(bdf, 0x44),
        }
    }
    fn configure(&mut self) {}
}

impl Display for StandardPCIDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let vendor_id = self.common.vendor_id.read();
        let class_id = self.common.class_id.read();
        let subclass_id = self.common.subclass_id.read();
        let descr = super::classification::description(class_id, subclass_id, 0);
        let vendor = super::vendors::vendor(vendor_id);
        write!(f, "{} {} at {}", descr, vendor, self.bdf)
    }
}
