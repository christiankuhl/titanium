pub mod keyboard;
pub mod mouse;
pub mod pci;
pub mod pic;
#[macro_use]
pub mod serial;
pub mod ahci;
mod blockdevice;
mod pit;
pub mod port;
pub mod vga;

pub fn init() {
    pit::init();
    pci::init();
    ahci::init();
    // vga::init();
}
