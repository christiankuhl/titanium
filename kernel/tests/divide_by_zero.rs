#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::testing::single_test)]
#![reexport_test_harness_main = "test_main"]

use core::arch::asm;
use kernel::*;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &kernel::MultibootInfo) {
    init(multiboot_info);
    test_main();
}

#[test_case]
fn divide_by_zero_panics() {
    unsafe {
        asm!("mov dx, 0; div dx");
    }
}
