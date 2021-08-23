#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

// TEMP:
#![allow(dead_code)]

extern crate alloc;

use kernel::*;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &MultibootInfo) -> ! {
    #[cfg(not(feature = "test_qemu_headless"))]
    debugprintln!("Entering Rust kernel...");

    init(multiboot_info);

    #[cfg(test)]
    test_main();

    let thread1 = multitasking::Thread::new(test1);
    let thread2 = multitasking::Thread::new(test2);
    {
        let mut scheduler = multitasking::SCHEDULER.lock();
        scheduler.add_thread(thread1);
        scheduler.add_thread(thread2);
        scheduler.start();
    }
    println!("Hello, world!\nHow are you on this most glorious of days?");
    enable_interrupts();
    
    idle();
}

fn test_runner(tests: &[&dyn testing::Testable]) {
    testing::test_runner_with_title(tests, "Testing basic boot...")
}

#[test_case]
fn basic_boot() {
    assert!(true);
}

fn test1() {
    loop {
        print!(".")
    }
}

fn test2() {
    loop {
        print!("+")
    }
}