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
    #[cfg(not(test))]
    debugprintln!("Entering Rust kernel...");

    init(multiboot_info);

    #[cfg(test)]
    test_main();

    enable_interrupts();
    
    println!("Hello, world!\nHow are you on this most glorious of days?");
    idle();
}

fn test_runner(tests: &[&dyn testing::Testable]) {
    testing::test_runner_with_title(tests, "Testing basic boot...")
}

#[test_case]
fn basic_boot() {
    assert!(true);
}
