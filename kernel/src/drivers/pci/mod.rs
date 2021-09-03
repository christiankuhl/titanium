use lazy_static::lazy_static;

mod classification;
mod controller;
mod vendors;

pub use classification::*;
use controller::PCIController;
pub use controller::*;

lazy_static! {
    pub static ref PCI: spin::Mutex<PCIController> = {
        let mut pci = PCIController::new();
        pci.discover();
        spin::Mutex::new(pci)
    };
}
