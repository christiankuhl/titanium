pub mod keyboard;
pub mod mouse;
pub mod pci;
pub mod pic;
#[macro_use]
pub mod serial;
pub mod ahci;
mod blockdevice;
pub mod port;
pub mod vga;
mod pit;

pub fn init() {
    pit::init();
    pci::init();
    ahci::init();
    // vga::init();
}
