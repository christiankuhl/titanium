#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![feature(asm)]
#![reexport_test_harness_main="test_main"]

use core::panic::PanicInfo;

use kernel::*;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &kernel::MultibootInfo) -> ! {
    init(multiboot_info);
    test_main();
    idle();
}

fn overflow_stack() {
    let temp = [1u8; 4096];
    println!("{:?}", temp);
    overflow_stack();
}

#[test_case]
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

fn panic(info: &PanicInfo) -> ! {
    kernel::testing::panic_handler(info);
}

fn test_runner(tests: &[&dyn testing::Testable]) {
    testing::test_runner_with_title(tests, "Testing exception handlers...")
}