#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![macro_use]

// TEMP:
#![allow(dead_code)]

extern crate alloc;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

use core::panic::PanicInfo;

mod multiboot;
mod drivers;
mod vga_buffer;
mod interrupt;
mod gdt;
mod memory;
mod allocator;
mod pci;
mod multitasking;
mod serial;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &multiboot::MultibootInfo) -> ! {
    debugprintln!("Entering Rust kernel...");

    debugprintln!("\nBootloader left us the following memory areas:");
    for region in multiboot_info.memory_map().iter() {
        debugprintln!("start: 0x{:0x}, length: {:}", region.base_addr, region.length);
    }

    debugprintln!("\nKernel sections:");
    for (idx, section) in multiboot_info.elf_sections().enumerate() {
        debugprintln!("    [{}] {} addr: 0x{:0x}, size: {:0x}, flags: 0x{:0x}", idx, section.name(), section.addr, section.size, section.flags);
    }

    debugprintln!("\nStart of kernel: 0x{:x}", multiboot_info.kernel_start());
    debugprintln!("End of kernel: 0x{:x}", multiboot_info.kernel_end());
    debugprintln!("Start of multiboot info section: 0x{:x}", multiboot_info.multiboot_start());
    debugprintln!("End of multiboot info section: 0x{:x}", multiboot_info.multiboot_end());
    
    debugprintln!("\nInitialising global descriptor table...");
    gdt::init();
    
    debugprintln!("\nInitialising interrupt descriptor table...");
    interrupt::init_idt();
    
    debugprintln!("\nInitialising interrupt controller...");
    unsafe { interrupt::PICS.lock().initialize() }; 
    



    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = unsafe {
    //     memory::BootInfoFrameAllocator::init(&boot_info.memory_regions)
    // };
    // allocator::init_heap(&mut mapper, &mut frame_allocator)
    //     .expect("Heap initialization failed!");
    // let mut device_manager = drivers::DriverManager::new();
    // let mut pci = pci::PCIController::new();
    // pci.enumerate();
    // {
    //     let mut taskmgr = multitasking::TASKMANAGER.lock();
    //     let task1 = multitasking::Task::new(test1);
    //     let task2 = multitasking::Task::new(test2);
    //     taskmgr.add_task(task1);
    //     taskmgr.add_task(task2);
    //     taskmgr.start();
    // }

    // let b = alloc::boxed::Box::new([0u8; 1usize << 21]);

    println!("Hello, world!\nHow are you on this most glorious of days?");

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

