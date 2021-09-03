use alloc::vec::Vec;
use core::sync::atomic::{compiler_fence, Ordering};
use lazy_static::lazy_static;

use super::pci::{BaseAddressRegister, PCIDevice};
use crate::asm::inb;
use crate::memory::VirtAddr;

const AHCI_BASE: VirtAddr = 0x400000;

mod controller;
mod port;
mod structs;

pub use controller::AHCIController;

lazy_static! {
    pub static ref AHCI_CONTROLLERS: spin::Mutex<Vec<AHCIController>> = {
        use super::pci::DeviceClassification::*;
        use super::pci::*;
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
    log!("\nInitializing AHCI storage controllers...");
    for ctrl in AHCI_CONTROLLERS.lock().iter_mut() {
        let irq = ctrl.interrupt_vector();
        super::pic::enable_interrupt(irq);
        unsafe {
            AHCI_CONTROLLERS.force_unlock();
            ctrl.identify_drives();
            AHCI_CONTROLLERS.lock();
        }
    }
}

fn fis_base(port_number: usize) -> VirtAddr {
    AHCI_BASE + (32 << 10) + (port_number << 8)
}

fn command_list_base(port_number: usize) -> VirtAddr {
    AHCI_BASE + (port_number << 10)
}

//// Command table descriptor base address
fn command_table_descriptor(port_number: usize, cmd_idx: usize) -> VirtAddr {
    AHCI_BASE + (40 << 10) + (port_number << 13) + ((cmd_idx as usize) << 8)
}

fn delay(microseconds: usize) {
    for _ in 0..microseconds {
        unsafe {
            inb(0x80);
        }
    }
}

fn full_memory_barrier() {
    compiler_fence(Ordering::AcqRel);
}
