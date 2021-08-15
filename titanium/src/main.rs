#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![macro_use]

extern crate alloc;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

use core::panic::PanicInfo;
use x86_64::VirtAddr;

mod drivers;
mod vga_buffer;
mod interrupt;
mod gdt;
// mod memory;
mod allocator;
mod pci;
mod multitasking;

fn test1() {
    loop {
        print!("A");
    }
}

fn test2() {
    loop {
        print!("B");
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {  
    println!("Hello, world!\nHow are you on this most glorious of days?");
    gdt::init();
    interrupt::init_idt();
    unsafe { interrupt::PICS.lock().initialize() }; 
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = unsafe {
    //     memory::BootInfoFrameAllocator::init(&boot_info.memory_regions)
    // };
    // allocator::init_heap(&mut mapper, &mut frame_allocator)
    //     .expect("Heap initialization failed!");
    let mut device_manager = drivers::DriverManager::new();
    let mut pci = pci::PCIController::new();
    pci.enumerate();
    // {
    //     let mut taskmgr = multitasking::TASKMANAGER.lock();
    //     let task1 = multitasking::Task::new(test1);
    //     let task2 = multitasking::Task::new(test2);
    //     taskmgr.add_task(task1);
    //     taskmgr.add_task(task2);
    //     taskmgr.start();
    // }

    // let b = alloc::boxed::Box::new([0u8; 1usize << 21]);

    x86_64::instructions::interrupts::enable();
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

