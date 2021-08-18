#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(asm)]
#![macro_use]

// TEMP:
#![allow(dead_code)]

extern crate alloc;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

use core::panic::PanicInfo;

mod multiboot;
mod drivers;
mod shell;
mod interrupt;
mod gdt;
mod memory;
mod pci;
mod multitasking;
mod serial;
mod asm_wrappers;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &multiboot::MultibootInfo) -> ! {
    debugprintln!("Entering Rust kernel...");

    debugprintln!("\nConfiguring physical memory...");
    memory::init(multiboot_info);
    
    debugprintln!("\nInitialising global descriptor table...");
    gdt::init();

    interrupt::init();

    let b = alloc::boxed::Box::new(42);

    println!("Hello, world!\nHow are you on this most glorious of days?");

    x86_64::instructions::interrupts::enable();
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

