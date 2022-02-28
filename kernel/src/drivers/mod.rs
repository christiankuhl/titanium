pub mod keyboard;
pub mod mouse;
pub mod pci;
pub mod pic;
#[macro_use]
pub mod serial;
pub mod ahci;
mod blockdevice;
pub mod port;

pub fn init() {
    pci::init();
    ahci::init();
}
