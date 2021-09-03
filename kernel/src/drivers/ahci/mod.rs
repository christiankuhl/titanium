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
        for (idx, dev) in pci_devices.drain(..).enumerate() {
            let mut ctrl = AHCIController::new(dev, idx);
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
        ctrl.finish_initialisation()
    }
}

//// Base address of AHCI MMIO structures
fn base_address(ctrl_number: usize) -> VirtAddr {
    AHCI_BASE + 3 * ctrl_number * (1 << 17)
}

//// FIS base address
fn fis_base(ctrl_number: usize, port_number: usize) -> VirtAddr {
    base_address(ctrl_number) + (32 << 10) + (port_number << 8)
}

//// Command list base address
fn command_list_base(ctrl_number: usize, port_number: usize) -> VirtAddr {
    base_address(ctrl_number) + (port_number << 10)
}

//// Command table descriptor base address
fn command_table_descriptor(ctrl_number: usize, port_number: usize, cmd_idx: usize) -> VirtAddr {
    base_address(ctrl_number) + (40 << 10) + (port_number << 13) + ((cmd_idx as usize) << 8)
}

//// Device metadata base address
fn metadata_address(ctrl_number: usize, port_number: usize) -> VirtAddr {
    base_address(ctrl_number) + 2 * (1 << 17) + (1 << 16) + 512 * port_number
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
