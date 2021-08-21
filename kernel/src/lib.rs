#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "run_tests"]
#![macro_use]

// TEMP:
#![allow(dead_code)]

extern crate alloc;

use core::panic::PanicInfo;

pub mod asm;
pub mod drivers;
pub mod interrupts;
pub mod memory;
pub mod multiboot;
pub mod multitasking;
pub mod shell;
pub mod testing;

pub use multiboot::MultibootInfo;
pub use asm::{idle, enable_interrupts};

pub fn init(multiboot_info: &multiboot::MultibootInfo) {
    interrupts::init();
    #[cfg(not(test))]
    debugprintln!("\nConfiguring physical memory...");
    memory::init(multiboot_info);
    drivers::init();
}

#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    idle();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
