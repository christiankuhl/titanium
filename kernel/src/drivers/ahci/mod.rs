use core::sync::atomic::{compiler_fence, Ordering};

use super::pci::{BaseAddressRegister, PCIDevice};
use crate::asm::inb;
use crate::memory::VirtAddr;

const AHCI_BASE: VirtAddr = 0x400000;

mod controller;
mod port;
mod structs;

pub use controller::AHCIController;

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
