use core::{
    mem::size_of,
    ops::{Deref, DerefMut},
};

use super::pci::{BaseAddressRegister, PCIDevice};
use crate::{
    graphics::Colour,
    memory::{allocate_identity_mapped, EntryFlags, Flags},
};
use alloc::{alloc::alloc, alloc::Layout, boxed::Box};
use lazy_static::lazy_static;

const QEMU: u16 = 0x1234;
const QEMU_VGA: u16 = 0x1111;
pub const SCREEN_WIDTH: u16 = 1024;
pub const SCREEN_HEIGHT: u16 = 768;

lazy_static! {
    pub static ref VGA: spin::Mutex<VideoGraphicsAdapter> = {
        use super::pci::DeviceClassification::*;
        use super::pci::*;
        let mut pci = PCI.lock();
        let class = DisplayController(Display::VGACompatible);
        let mut pci_devices = pci.get_devices(class);
        let vga = pci_devices.pop().expect("No VGA compatible graphics card found!");
        if !VideoGraphicsAdapter::is_compatible(&vga) {
            panic!("No compatible VGA found!");
        }
        spin::Mutex::new(VideoGraphicsAdapter::new(vga))
    };
}

pub struct VideoGraphicsAdapter {
    device: Box<PCIDevice>,
    registers: &'static mut QEMUDisplayRegisters,
    framebuffer: &'static mut FrameBuffer,
}

impl VideoGraphicsAdapter {
    fn new(pci: Box<PCIDevice>) -> Self {
        let mmio_ptr = if let BaseAddressRegister::MemoryMapped(mmio_bar) = (*pci).bar[2] {
            mmio_bar.base_address()
        } else {
            panic!("VGA malfunction!")
        };
        allocate_identity_mapped(mmio_ptr, size_of::<QEMUDisplayRegisters>(), EntryFlags::WRITABLE);
        let framebuffer_ptr = if let BaseAddressRegister::MemoryMapped(mmio_bar) = (*pci).bar[0] {
            mmio_bar.base_address()
        } else {
            panic!("VGA malfunction!")
        };
        allocate_identity_mapped(framebuffer_ptr, 4 * SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize, EntryFlags::WRITABLE);
        Self {
            device: pci,
            registers: unsafe { &mut *(mmio_ptr as *mut QEMUDisplayRegisters) },
            framebuffer: unsafe { &mut *(framebuffer_ptr as *mut FrameBuffer) },
        }
    }
    fn is_compatible(pci: &Box<PCIDevice>) -> bool {
        let vendor_id = (*pci).common.vendor_id.read();
        let device_id = (*pci).common.device_id.read();
        return vendor_id == QEMU && device_id == QEMU_VGA;
    }
    fn init(&mut self) {
        log!("\nInitializing graphics card at {}", self.device.bdf);
        self.framebuffer.clear();
        self.unblank();
        self.set_resolution(SCREEN_WIDTH, SCREEN_HEIGHT);
    }
    fn set_resolution(&mut self, width: u16, height: u16) {
        log!("... VGA resolution set to - {}x{}", width, height);
        self.registers.enable = 0;
        self.registers.xres = width;
        self.registers.yres = height;
        self.registers.virt_width = width;
        self.registers.virt_height = height * 2;
        self.registers.bpp = 32;
        self.registers.enable = 0x01 | 0x41;
        self.registers.bank = 0;
    }
    fn unblank(&mut self) {
        self.registers.vga_ioports[0] = 0x20;
    }
    #[inline]
    pub fn flip(&mut self, buffer: &ScreenBuffer) {
        self.framebuffer.flip(buffer);
    }
}

type _ScreenBuffer = [Colour; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize];

#[repr(C)]
pub struct ScreenBuffer(_ScreenBuffer);

impl ScreenBuffer {
    pub fn new() -> Box<Self> {
        unsafe {
            let ptr = alloc(Layout::new::<_ScreenBuffer>()) as *mut Self;
            let mut result = Box::from_raw(ptr);
            (*result).fill(Colour::black());
            result
        }
    }
    pub fn clear(&mut self) {
        self.0.fill(Colour::black());
    }
}

impl<Idx> core::ops::Index<Idx> for ScreenBuffer
where
    Idx: core::slice::SliceIndex<[Colour]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index]
    }
}

impl<Idx> core::ops::IndexMut<Idx> for ScreenBuffer
where
    Idx: core::slice::SliceIndex<[Colour]>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Deref for ScreenBuffer {
    type Target = _ScreenBuffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScreenBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[repr(C, packed)]
struct FrameBuffer {
    buffer: ScreenBuffer,
}

impl FrameBuffer {
    fn clear(&mut self) {
        self.buffer.0.fill(Colour::black());
    }
    #[inline]
    fn flip(&mut self, buffer: &ScreenBuffer) {
        self.buffer.0.copy_from_slice(&buffer.0);
    }
}

#[repr(C, packed)]
struct QEMUDisplayRegisters {
    edid_data: [u8; 1024],
    vga_ioports: [u16; 16],
    _reserved: [u8; 224],
    index_id: u16,
    xres: u16,
    yres: u16,
    bpp: u16,
    enable: u16,
    bank: u16,
    virt_width: u16,
    virt_height: u16,
    x_offset: u16,
    y_offset: u16,
}

pub fn init() {
    let mut vga = VGA.lock();
    vga.init();
}
