pub mod keyboard;
pub mod mouse;
pub mod pci;
pub mod pic;
#[macro_use]
pub mod serial;
pub mod ahci;

use ahci::AHCIController;

use crate::println;

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
    use pci::DeviceClassification::*;
    use pci::*;
    let mut pci = pci::init();
    let class = MassStorageController(MassStorage::SerialATA);
    let mut ahci_controllers = pci.get_devices(class);
    log!("\nLooking for AHCI compatible storage devices...");
    for dev in ahci_controllers.drain(..) {
        log!("{}:", dev);
        let mut ctrl = AHCIController::new(dev);
        ctrl.initialize();
        let drives = ctrl.enumerate_ports();
        for mut d in drives {
            d.identify();
        }
    }
}
