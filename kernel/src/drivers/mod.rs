pub mod keyboard;
pub mod mouse;
pub mod pci;
pub mod pic;
#[macro_use]
pub mod serial;

trait Driver {
    fn init(&mut self);
    fn reset(&mut self) -> u8;
    fn deactivate(&mut self);
}

pub struct DriverManager {}

impl DriverManager {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn init() {
    log!("\nLooking for PCI devices...");
    pci::init();
}
