#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(asm)]
#![feature(naked_functions)]
#![macro_use]
// TEMP:
// #![allow(dead_code)]

extern crate alloc;

use core::panic::PanicInfo;

mod asm_wrappers;
mod drivers;
mod gdt;
mod interrupt;
mod interrupts;
mod memory;
mod multiboot;
mod multitasking;
mod serial;
mod shell;

use asm_wrappers::idle;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &multiboot::MultibootInfo) -> ! {
    debugprintln!("Entering Rust kernel...");

    debugprintln!("\nInitialising global descriptor table...");
    gdt::init();

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

    x86_64::instructions::interrupts::enable();

    println!("Hello, world!\nHow are you on this most glorious of days?");

    idle();
}

fn overflow_stack() {
    let temp = [1u8; 4096];
    println!("{:?}", temp);
    overflow_stack();
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx");
    }
}

fn page_fault() {
    unsafe {
        *(0xdeadbeef as *mut u8) = 3;
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    idle();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
