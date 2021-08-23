
pub mod mouse;
pub mod keyboard;
pub mod pic;
pub mod pci;
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
    // let mut device_manager = drivers::DriverManager::new();
    #[cfg(not(feature = "test_qemu_headless"))]
    debugprintln!("\nLooking for PCI devices...");
    let mut pci = pci::PCIController::new();
    pci.enumerate();
}