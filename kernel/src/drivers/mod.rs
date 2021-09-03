use alloc::vec::Vec;
use lazy_static::lazy_static;

pub mod keyboard;
pub mod mouse;
pub mod pci;
pub mod pic;
#[macro_use]
pub mod serial;
pub mod ahci;
#[macro_use]
mod macros;

use ahci::AHCIController;

lazy_static! {
    pub static ref AHCI_CONTROLLERS: spin::Mutex<Vec<AHCIController>> = {
        use pci::DeviceClassification::*;
        use pci::*;
        let mut pci = PCI.lock();
        let class = MassStorageController(MassStorage::SerialATA);
        let mut pci_devices = pci.get_devices(class);
        let mut ahci_controllers = Vec::new();
        for dev in pci_devices.drain(..) {
            let mut ctrl = AHCIController::new(dev);
            ctrl.initialize();
            ahci_controllers.push(ctrl);
        }
        spin::Mutex::new(ahci_controllers)
    };
}

pub fn init() {
    for ctrl in AHCI_CONTROLLERS.lock().iter_mut() {
        let irq = ctrl.interrupt_vector();
        pic::enable_interrupt(irq);
        unsafe {
            AHCI_CONTROLLERS.force_unlock();
            ctrl.identify_drives();
            AHCI_CONTROLLERS.lock();
        }
    }
}
