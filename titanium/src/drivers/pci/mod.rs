use crate::debugprintln;
use core::mem::size_of;
use core::{marker::PhantomData, ops::BitAnd, ops::BitOr, ops::Shl, ops::Shr};
use x86_64::instructions::port::Port;

mod devices;
mod vendors;

pub struct PCIController {
    ctrl_port: Port<u32>,
    data_port: Port<u32>,
}

impl PCIController {
    pub fn new() -> Self {
        Self { ctrl_port: Port::new(0xcf8), data_port: Port::new(0xcfc) }
    }
    fn read(&mut self, bdf: BDF, offset: u8) -> RegisterValue {
        unsafe {
            self.ctrl_port.write(bdf.id(offset));
            RegisterValue(self.data_port.read() >> (8 * (offset % 4)))
        }
    }
    fn write(&mut self, bdf: BDF, offset: u8, value: RegisterValue) {
        unsafe {
            self.ctrl_port.write(bdf.id(offset));
            self.data_port.write(value.into());
        }
    }

    pub fn enumerate(&mut self) {
        let host_device =
            PCIDevice::new(self, BDF { bus: 0, device: 0, function: 0 }).unwrap_or_else(|| panic!("No PCI host device found!"));
        if !host_device.common().multifunction {
            self.enumerate_bus(0)
        } else {
            for function in 0..8 {
                let bdf = BDF { bus: 0, device: 0, function };
                let maybe_device = PCIDevice::new(self, bdf);
                if let Some(device) = maybe_device {
                    let vendor_id: u32 = device.common().vendor_id.read(self).into();
                    if vendor_id != 0xffff {
                        break;
                    }
                }
                self.enumerate_bus(function);
            }
        }
    }

    fn enumerate_bus(&mut self, bus: u8) {
        for device in 0..32 {
            self.enumerate_device(bus, device)
        }
    }

    fn enumerate_device(&mut self, bus: u8, device: u8) {
        let bdf = BDF { bus, device, function: 0 };
        if let Some(mut pcidevice) = PCIDevice::new(self, bdf) {
            let header = pcidevice.common();
            let vendor_id: u32 = header.vendor_id.read(self).into();
            if vendor_id == 0xffff {
                return;
            }
            self.check_function(&mut pcidevice);
            if header.multifunction {
                for function in 1..8 {
                    let bdf = BDF { bus, device, function };
                    if let Some(mut pcidevice) = PCIDevice::new(self, bdf) {
                        let vendor_id: u32 = pcidevice.common().vendor_id.read(self).into();
                        if vendor_id == 0xffff {
                            continue;
                        }
                        self.check_function(&mut pcidevice);
                    }
                }
            }
        }
    }

    fn check_function(&mut self, pcidevice: &mut PCIDevice) {
        let class_id = pcidevice.common().class_id.read(self);
        let subclass_id = pcidevice.common().subclass_id.read(self);
        match pcidevice {
            PCIDevice::PCIBridge(bridge) => {
                debugprintln!("{}", devices::description(class_id, subclass_id, 0));
                let secondary_bus = bridge.secondary_bus.read(self);
                self.enumerate_bus(secondary_bus);
            }
            _ => {
                let vendor_id = pcidevice.common().vendor_id.read(self);
                debugprintln!(
                    "    {} {} at {:?}",
                    vendors::vendor(vendor_id),
                    devices::description(class_id, subclass_id, 0),
                    pcidevice.bdf()
                );
                pcidevice.configure(self);
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct IOBAR(u32);

impl IOBAR {
    pub fn port_number(&self) -> u32 {
        self.0 & 0xfffffffc
    }
}

#[derive(Copy, Clone, Debug)]
struct MemoryMappedBAR(u32, u32, u64);

impl MemoryMappedBAR {
    pub fn prefetchable(&self) -> bool {
        self.0 & 0x8 > 0
    }
    pub fn size(&self) -> BARSize {
        match self.0 % 8 {
            0 => BARSize::Bit32,
            1 => BARSize::Bit20,
            _ => BARSize::Bit64,
        }
    }
    pub fn mask(&self) -> u64 {
        self.2
    }
    pub fn base_address(&self) -> usize {
        match self.size() {
            BARSize::Bit20 => (self.0 & 0xfffff0) as usize,
            BARSize::Bit32 => (self.0 & 0xfffff0) as usize,
            BARSize::Bit64 => ((self.0 & 0xfffff0) | (self.1 << 32)) as usize,
        }
    }
    pub fn configure(&mut self, bdf: BDF, offset: u8, pci: &mut PCIController) {
        pci.write(bdf, offset, RegisterValue(0xfffffff0));
        let temp: u32 = pci.read(bdf, offset).into();
        self.2 = (temp & 0xfffffff0) as u64;
        pci.write(bdf, offset, RegisterValue(self.0));
        if self.size() == BARSize::Bit64 {
            pci.write(bdf, offset + 4, RegisterValue(0xffffffff));
            let temp: u32 = pci.read(bdf, offset + 4).into();
            self.2 += (temp as u64) << 32;
            pci.write(bdf, offset, RegisterValue(self.0));
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum BaseAddressRegister {
    IO(IOBAR),
    MemoryMapped(MemoryMappedBAR),
    Unused,
}

impl BaseAddressRegister {
    fn new(bar: u32, extension: u32) -> Self {
        if bar == 0 {
            return Self::Unused;
        }
        if bar % 2 == 0 {
            Self::IO(IOBAR(bar))
        } else {
            Self::MemoryMapped(MemoryMappedBAR(bar, extension, 0))
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct BaseAddressRegisters([BaseAddressRegister; 6]);

impl BaseAddressRegisters {
    pub fn new() -> Self {
        use BaseAddressRegister::Unused;
        BaseAddressRegisters([Unused, Unused, Unused, Unused, Unused, Unused])
    }
    pub fn configure(&mut self, device: &PCIDevice, pci: &mut PCIController) {
        let bdf = device.bdf();
        let max_bars = match device {
            PCIDevice::Standard(_) => 6,
            PCIDevice::PCIBridge(_) => 2,
            PCIDevice::PCICardBus(_) => 0,
        };
        let mut skip = false;
        for j in 0..max_bars {
            if skip {
                skip = false;
                continue;
            }
            let reg: u32 = pci.read(bdf, 4 * j + 0x10).into();
            let ext: u32 = pci.read(bdf, 4 * j + 0x14).into();
            let bar = BaseAddressRegister::new(reg, ext);
            match bar {
                BaseAddressRegister::IO(_) => {
                    self.0[j as usize] = bar;
                }
                BaseAddressRegister::MemoryMapped(mut b) => {
                    b.configure(bdf, 4 * j + 0x10, pci);
                    self.0[j as usize] = BaseAddressRegister::MemoryMapped(b);
                    if b.size() == BARSize::Bit64 {
                        skip = true;
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(PartialEq)]
enum BARSize {
    Bit32,
    Bit20,
    Bit64,
}

#[derive(Copy, Clone, Debug)]
enum PCIDevice {
    Standard(StandardPCIDevice),
    PCIBridge(PCIBridge),
    PCICardBus(PCICardBusBridge),
}

impl PCIDevice {
    pub fn new(pci: &mut PCIController, bdf: BDF) -> Option<Self> {
        let header_type: u32 = pci.read(bdf, 0xc).into();
        let multifunction = header_type & 0x800000 > 0;
        let common = CommonPCIHeader::new(bdf, multifunction);
        match (header_type & 0x7f0000) >> 16 {
            0 => Some(Self::Standard(StandardPCIDevice::new(bdf, common))),
            1 => Some(Self::PCIBridge(PCIBridge::new(bdf, common))),
            2 => Some(Self::PCICardBus(PCICardBusBridge::new(bdf, common))),
            _ => None,
            // _ => panic!("Unknown PCI header type {:01x}!", (header_type & 0x7f0000) >> 16)
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
    pub fn configure(&mut self, pci: &mut PCIController) {
        match self {
            Self::Standard(device) => device.configure(pci),
            Self::PCIBridge(device) => device.configure(pci),
            Self::PCICardBus(device) => device.configure(pci),
        }
    }
}

struct RegisterValue(u32);

impl From<u8> for RegisterValue {
    fn from(value: u8) -> RegisterValue {
        RegisterValue(value as u32)
    }
}

impl From<u16> for RegisterValue {
    fn from(value: u16) -> RegisterValue {
        RegisterValue(value as u32)
    }
}

impl From<u32> for RegisterValue {
    fn from(value: u32) -> RegisterValue {
        RegisterValue(value)
    }
}

impl Into<u8> for RegisterValue {
    fn into(self) -> u8 {
        self.0 as u8
    }
}

impl Into<u16> for RegisterValue {
    fn into(self) -> u16 {
        self.0 as u16
    }
}

impl Into<u32> for RegisterValue {
    fn into(self) -> u32 {
        self.0
    }
}

impl Shr for RegisterValue {
    type Output = RegisterValue;
    fn shr(self, rhs: Self) -> Self::Output {
        RegisterValue(self.0 >> rhs.0)
    }
}

impl Shl for RegisterValue {
    type Output = RegisterValue;
    fn shl(self, rhs: Self) -> Self::Output {
        RegisterValue(self.0 << rhs.0)
    }
}

impl BitAnd for RegisterValue {
    type Output = RegisterValue;
    fn bitand(self, rhs: Self) -> Self::Output {
        RegisterValue(self.0 & rhs.0)
    }
}

impl BitOr for RegisterValue {
    type Output = RegisterValue;
    fn bitor(self, rhs: Self) -> Self::Output {
        RegisterValue(self.0 | rhs.0)
    }
}

#[derive(Copy, Clone, Debug)]
struct Register<T>(u8, BDF, PhantomData<T>);

impl<T> Register<T>
where
    RegisterValue: From<T>,
    RegisterValue: Into<T>,
{
    pub fn new(bdf: BDF, offset: u8) -> Self {
        Register(offset, bdf, PhantomData)
    }
    pub fn write(&self, value: T, pci: &mut PCIController) {
        let bdf = self.1;
        let aligned_offset = self.0 & 0xfc;
        let register_value = pci.read(bdf, aligned_offset);
        let shift = 32 - 8 * (self.0 - aligned_offset) as usize - 8 * size_of::<T>();
        let mask = 0xffffffff - ((0x1 << (32 - 8 * (self.0 - aligned_offset))) - 1) + ((0x1 << shift) - 1);
        let write_value = register_value & RegisterValue(mask) | RegisterValue::from(value) << RegisterValue(shift as u32);
        pci.write(bdf, self.0, write_value);
    }
    pub fn read(&self, pci: &mut PCIController) -> T {
        let bdf = self.1;
        let aligned_offset = self.0 & 0xfc;
        let aligned_result = pci.read(bdf, aligned_offset);
        let shift = 32 - 8 * (self.0 - aligned_offset) as usize - 8 * size_of::<T>();
        (aligned_result >> RegisterValue(shift as u32)).into()
    }
}

#[derive(Copy, Clone, Debug)]
struct CommonPCIHeader {
    device_id: Register<u16>,
    vendor_id: Register<u16>,
    status: Register<u16>,
    command: Register<u16>,
    class_id: Register<u8>,
    subclass_id: Register<u8>,
    interface_id: Register<u8>,
    revision: Register<u8>,
    bist: Register<u8>,
    multifunction: bool,
    latency_timer: Register<u8>,
    cache_line_size: Register<u8>,
}

impl CommonPCIHeader {
    pub fn new(bdf: BDF, multifunction: bool) -> Self {
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
struct StandardPCIDevice {
    bdf: BDF,
    common: CommonPCIHeader,
    bar: BaseAddressRegisters,
    cis_pointer: Register<u32>,
    subsystem_id: Register<u16>,
    subsystem_vendor_id: Register<u16>,
    expansion_base: Register<u32>,
    cap_pointer: Register<u8>,
    max_latency: Register<u8>,
    min_grant: Register<u8>,
    interrupt_pin: Register<u8>,
    interrupt_line: Register<u8>,
}

impl StandardPCIDevice {
    pub fn new(bdf: BDF, common: CommonPCIHeader) -> Self {
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
    pub fn configure(&mut self, pci: &mut PCIController) {
        self.bar.configure(&PCIDevice::Standard(*self), pci);
    }
}

#[derive(Copy, Clone, Debug)]
struct PCIBridge {
    bdf: BDF,
    common: CommonPCIHeader,
    bar: BaseAddressRegisters,
    sec_latency_timer: Register<u8>,
    subord_bus: Register<u8>,
    secondary_bus: Register<u8>,
    primary_bus: Register<u8>,
    secondary_status: Register<u16>,
    io_limit: u32,
    io_base: u32,
    memory_limit: Register<u16>,
    memory_base: Register<u16>,
    prefetchable_limit: usize,
    prefetchable_base: usize,
    cap_pointer: Register<u8>,
    expansion_base: Register<u32>,
    bridge_ctrl: Register<u16>,
    interrupt_pin: Register<u8>,
    interrupt_line: Register<u8>,
}

impl PCIBridge {
    pub fn new(bdf: BDF, common: CommonPCIHeader) -> Self {
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
    pub fn configure(&mut self, pci: &mut PCIController) {
        self.bar.configure(&PCIDevice::PCIBridge(*self), pci);
    }
}

#[derive(Copy, Clone, Debug)]
struct PCICardBusBridge {
    bdf: BDF,
    common: CommonPCIHeader,
    cardbus_base: Register<u32>,
    secondary_status: Register<u16>,
    cap_offset: Register<u8>,
    cardbus_latency_timer: Register<u8>,
    subordinate_bus: Register<u8>,
    cardbus: Register<u8>,
    pci_bus: Register<u8>,
    memory_base: [u32; 2],
    memory_limit: [u32; 2],
    io_base: [u32; 2],
    io_limit: [u32; 2],
    bridge_ctrl: Register<u16>,
    interrupt_pin: Register<u8>,
    interrupt_line: Register<u8>,
    subsystem_vendor_id: Register<u16>,
    subsystem_device_id: Register<u16>,
    legacy_base: Register<u32>,
}

impl PCICardBusBridge {
    pub fn new(bdf: BDF, common: CommonPCIHeader) -> Self {
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
    pub fn configure(&mut self, _pci: &mut PCIController) {}
}

#[derive(Copy, Clone, Debug)]
struct BDF {
    bus: u8,
    device: u8,
    function: u8,
}

impl BDF {
    pub fn id(&self, offset: u8) -> u32 {
        0x1 << 31
            | (((self.bus & 0xff) as u32) << 16)
            | (((self.device & 0x1f) as u32) << 11)
            | (((self.function & 0x7) as u32) << 8)
            | ((offset & 0xfc) as u32)
    }
}
