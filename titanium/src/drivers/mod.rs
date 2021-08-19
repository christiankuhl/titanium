use crate::debugprintln;

pub mod mouse;
pub mod keyboard;
pub mod pic;
pub mod pci;

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
    // let mut device_manager = drivers::DriverManager::new();
    debugprintln!("\nLooking for PCI devices...");
    let mut pci = pci::PCIController::new();
    pci.enumerate();
}