#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::testing::test_runner)]
#![feature(asm)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &kernel::MultibootInfo) -> ! {
    kernel::debugprintln!("--------------------------------------------------------------------------------");
    kernel::debugprintln!("| Testing exception handlers...                                                |");
    kernel::debugprintln!("--------------------------------------------------------------------------------");
    kernel::init(multiboot_info);
    test_main();
    kernel::idle();
}

fn overflow_stack() {
    let temp = [1u8; 4096];
    kernel::println!("{:?}", temp);
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