#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(naked_functions)]
#![feature(asm_sym)]
#![feature(custom_test_frameworks)]
#![feature(arbitrary_enum_discriminant)]
#![feature(drain_filter)]
#![feature(int_log)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "run_tests"]
#![macro_use]
#![allow(dead_code)]

extern crate alloc;

use core::panic::PanicInfo;

pub mod asm;
pub mod drivers;
#[macro_use]
pub mod interrupts;
pub mod filesystem;
pub mod graphics;
pub mod memory;
pub mod multiboot;
pub mod multitasking;
pub mod shell;
pub mod syscalls;
pub mod testing;
pub mod time;

pub use asm::{enable_interrupts, idle};
pub use multiboot::MultibootInfo;

pub fn init(multiboot_info: &multiboot::MultibootInfo) {
    interrupts::init();
    log!("\nConfiguring physical memory...");
    memory::init(multiboot_info);
    time::init();
    enable_interrupts();
    drivers::init();
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    #[cfg(not(any(feature = "qemu_test", feature = "qemu_test_should_panic")))]
    {
        println!("{}", info);
        idle();
    }
    #[cfg(feature = "qemu_test")]
    testing::panic_handler(info);
    #[cfg(feature = "qemu_test_should_panic")]
    testing::should_panic(info);
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
