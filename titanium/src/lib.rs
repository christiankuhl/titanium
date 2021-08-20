#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(asm)]
#![feature(naked_functions)]
#![macro_use]

// TEMP:
#![allow(dead_code)]

extern crate alloc;

use core::panic::PanicInfo;

mod asm;
mod drivers;
mod interrupts;
mod memory;
mod multiboot;
mod multitasking;
mod shell;

pub use multiboot::MultibootInfo;
use asm::{idle, enable_interrupts};
pub use drivers::serial::_print as _serial_print;

pub fn init(multiboot_info: &multiboot::MultibootInfo) -> ! {
    interrupts::init();
    
    debugprintln!("\nConfiguring physical memory...");
    memory::init(multiboot_info);

    drivers::init();

    // {
    //     let mut taskmgr = multitasking::TASKMANAGER.lock();
    //     let task1 = multitasking::Task::new(test1);
    //     let task2 = multitasking::Task::new(test2);
    //     taskmgr.add_task(task1);
    //     taskmgr.add_task(task2);
    //     taskmgr.start();
    // }

    println!("Hello, world!\nHow are you on this most glorious of days?");
    enable_interrupts();
    idle();
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    asm::idle();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
