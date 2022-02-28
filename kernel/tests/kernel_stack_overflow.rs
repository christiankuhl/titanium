#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::testing::single_test)]
#![reexport_test_harness_main = "test_main"]

use kernel::*;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &kernel::MultibootInfo) {
    init(multiboot_info);
    test_main();
}

#[test_case]
fn guard_page_present() {
    unsafe {
        *(0x169000 as *mut u8) = 42;
    }
}
