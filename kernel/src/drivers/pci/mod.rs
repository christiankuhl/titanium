mod classification;
mod controller;
mod vendors;

use controller::PCIController;

pub fn init() {
    let mut pci: PCIController = PCIController::new();
    pci.discover();
}
