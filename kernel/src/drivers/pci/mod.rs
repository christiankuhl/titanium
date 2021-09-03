use lazy_static::lazy_static;

use crate::log;

mod classification;
mod controller;
mod vendors;

pub use classification::*;
use controller::PCIController;
pub use controller::*;

lazy_static! {
    pub static ref PCI: spin::Mutex<PCIController> = { spin::Mutex::new(PCIController::new()) };
}

pub fn init() {
    log!("\nLooking for PCI devices...");
    PCI.lock().discover();
}
