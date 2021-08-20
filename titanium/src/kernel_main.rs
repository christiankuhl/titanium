#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &kernel::MultibootInfo) -> ! {
    kernel::debugprintln!("Entering Rust kernel...");
    kernel::init(multiboot_info);
}