use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Display;
use core::mem::size_of;
use core::{marker::PhantomData, ops::BitAnd, ops::BitOr, ops::Index, ops::Shl, ops::Shr};

use crate::asm::{inl, outl};
use crate::log;

use super::{classification, vendors};
pub use classification::DeviceClassification;

mod devices;

pub use devices::StandardPCIDevice as PCIDevice;

const CTRL_PORT: u16 = 0xcf8;
const DATA_PORT: u16 = 0xcfc;

pub struct PCIController {
    devices: Vec<Box<PCIDevice>>,
}

impl PCIController {
    pub fn new() -> Self {
        Self { devices: Vec::new() }
    }
    pub fn get_devices(&mut self, class: DeviceClassification) -> Vec<Box<PCIDevice>> {
        self.devices.drain_filter(|dev| (dev.common.class_id.read(), dev.common.subclass_id.read()) == class.as_raw()).collect()
    }
    pub fn discover(&mut self) {
        let host_device = devices::PCIDevice::new(BDF { bus: 0, device: 0, function: 0 })
            .unwrap_or_else(|| panic!("No PCI host device found!"));
        if !host_device.common().multifunction {
            self.enumerate_bus(0)
        } else {
            for function in 0..8 {
                let bdf = BDF { bus: 0, device: 0, function };
                let maybe_device = devices::PCIDevice::new(bdf);
                if let Some(device) = maybe_device {
                    let vendor_id = device.common().vendor_id.read();
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
        if let Some(mut pcidevice) = devices::PCIDevice::new(bdf) {
            let header = pcidevice.common();
            let vendor_id = header.vendor_id.read();
            if vendor_id == 0xffff {
                return;
            }
            self.check_function(&mut pcidevice);
            if header.multifunction {
                for function in 1..8 {
                    let bdf = BDF { bus, device, function };
                    if let Some(mut pcidevice) = devices::PCIDevice::new(bdf) {
                        let vendor_id = pcidevice.common().vendor_id.read();
                        if vendor_id == 0xffff {
                            continue;
                        }
                        self.check_function(&mut pcidevice);
                    }
                }
            }
        }
    }

    fn check_function(&mut self, pcidevice: &mut devices::PCIDevice) {
        let class_id = pcidevice.common().class_id.read();
        let subclass_id = pcidevice.common().subclass_id.read();
        match pcidevice {
            devices::PCIDevice::PCIBridge(bridge) => {
                log!("{}", classification::description(class_id, subclass_id, 0));
                let secondary_bus = bridge.secondary_bus.read();
                self.enumerate_bus(secondary_bus);
            }
            devices::PCIDevice::Standard(device) => {
                log!("    {}", device);
                device.configure();
                self.devices.push(Box::new(*device))
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct IOBAR(u32);

impl IOBAR {
    pub fn port_number(&self) -> u32 {
        self.0 & 0xfffffffc
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MemoryMappedBAR(u32, u32, u64);

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
            BARSize::Bit32 => (self.0 & 0xfffffff0) as usize,
            BARSize::Bit64 => ((self.0 & 0xfffffff0) as usize | ((self.1 as usize) << 32)),
        }
    }
    pub fn configure(&mut self, bdf: BDF, offset: u8) {
        let pci = Register::<u32>::new(bdf, offset);
        pci.write(0xfffffff0);
        let temp: u32 = pci.read();
        self.2 = (temp & 0xfffffff0) as u64;
        pci.write(self.0);
        if self.size() == BARSize::Bit64 {
            let next = Register::<u32>::new(bdf, offset + 4);
            next.write(0xffffffff);
            let temp: u32 = pci.read();
            self.2 += (temp as u64) << 32;
            pci.write(self.0);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BaseAddressRegister {
    IO(IOBAR),
    MemoryMapped(MemoryMappedBAR),
    Unused,
}

impl BaseAddressRegister {
    fn new(bar: u32, extension: u32) -> Self {
        if bar == 0 {
            return Self::Unused;
        }
        if bar % 2 == 1 {
            Self::IO(IOBAR(bar))
        } else {
            Self::MemoryMapped(MemoryMappedBAR(bar, extension, 0))
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BaseAddressRegisters([BaseAddressRegister; 6]);

impl BaseAddressRegisters {
    fn new() -> Self {
        use BaseAddressRegister::Unused;
        BaseAddressRegisters([Unused, Unused, Unused, Unused, Unused, Unused])
    }
    fn configure(&mut self, device: &devices::PCIDevice) {
        let bdf = device.bdf();
        let max_bars = match device {
            devices::PCIDevice::Standard(_) => 6,
            devices::PCIDevice::PCIBridge(_) => 2,
            devices::PCIDevice::PCICardBus(_) => 0,
        };
        let mut skip = false;
        for j in 0..max_bars {
            if skip {
                skip = false;
                continue;
            }
            let pci = Register::<u32>::new(bdf, 4 * j + 0x10);
            let next = Register::<u32>::new(bdf, 4 * j + 0x14);
            let reg: u32 = pci.read();
            let ext: u32 = next.read();
            let bar = BaseAddressRegister::new(reg, ext);
            match bar {
                BaseAddressRegister::IO(_) => {
                    self.0[j as usize] = bar;
                }
                BaseAddressRegister::MemoryMapped(mut b) => {
                    b.configure(bdf, 4 * j + 0x10);
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

impl Index<usize> for BaseAddressRegisters {
    type Output = BaseAddressRegister;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(PartialEq, Debug)]
pub enum BARSize {
    Bit32,
    Bit20,
    Bit64,
}

pub struct RegisterValue(pub u32);

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
pub struct Register<T>(u8, BDF, PhantomData<T>);

impl<T> Register<T>
where
    RegisterValue: From<T>,
    RegisterValue: Into<T>,
{
    fn new(bdf: BDF, offset: u8) -> Self {
        Register(offset, bdf, PhantomData)
    }
    pub fn write(&self, value: T) {
        let bdf = self.1;
        let aligned_offset = self.0 & 0xfc;
        let register_value = self.read_unsafe(bdf, aligned_offset);
        let shift = 32 - 8 * (self.0 - aligned_offset) as usize - 8 * size_of::<T>();
        let mask: u32 = 0xffffffff - ((0x1u64 << (32 - 8 * (self.0 - aligned_offset))) - 1) as u32 + ((0x1 << shift) - 1);
        let write_value = register_value & RegisterValue(mask) | RegisterValue::from(value) << RegisterValue(shift as u32);
        self.write_unsafe(bdf, self.0, write_value);
    }
    pub fn read(&self) -> T {
        let bdf = self.1;
        let aligned_offset = self.0 & 0xfc;
        let aligned_result = self.read_unsafe(bdf, aligned_offset);
        let shift = 32 - 8 * (self.0 - aligned_offset) as usize - 8 * size_of::<T>();
        (aligned_result >> RegisterValue(shift as u32)).into()
    }
    fn read_unsafe(&self, bdf: BDF, offset: u8) -> RegisterValue {
        unsafe { outl(CTRL_PORT, bdf.id(offset)) }
        RegisterValue(unsafe { inl(DATA_PORT) } >> (8 * (offset % 4)))
    }
    fn write_unsafe(&self, bdf: BDF, offset: u8, value: RegisterValue) {
        unsafe {
            outl(CTRL_PORT, bdf.id(offset));
            outl(DATA_PORT, value.0);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BDF {
    bus: u8,
    device: u8,
    function: u8,
}

impl BDF {
    #[inline]
    fn id(&self, offset: u8) -> u32 {
        0x1 << 31
            | (((self.bus & 0xff) as u32) << 16)
            | (((self.device & 0x1f) as u32) << 11)
            | (((self.function & 0x7) as u32) << 8)
            | ((offset & 0xfc) as u32)
    }
}

impl Display for BDF {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "bus {}, device {}, function {}", self.bus, self.device, self.function)
    }
}
