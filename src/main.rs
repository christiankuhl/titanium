#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
// #![feature(custom_test_frameworks)]
#![feature(const_mut_refs)]
#![macro_use]
// #![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}


use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

mod drivers;
mod vga_buffer;
mod interrupt;
mod gdt;
mod memory;
mod allocator;
mod pci;

entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello, world!\nHow are you on this most glorious of days?");
    gdt::init();
    interrupt::init_idt();
    unsafe { interrupt::PICS.lock().initialize() }; 
    x86_64::instructions::interrupts::enable(); 
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    let mut device_manager = drivers::DriverManager::new();
    let mut pci = pci::PCIController::new();
    pci.enumerate();

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