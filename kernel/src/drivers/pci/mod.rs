mod classification;
mod controller;
mod vendors;

pub use classification::*;
use controller::PCIController;
pub use controller::*;

pub fn init() -> PCIController {
    let mut pci = PCIController::new();
    pci.discover();
    pci
}
