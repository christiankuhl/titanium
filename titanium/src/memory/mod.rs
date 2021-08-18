use crate::{asm_wrappers::{enable_nxe_bit, enable_write_protect_bit}, debugprintln, memory::paging::remap_kernel, multiboot::MultibootInfo};

mod region_frame_allocator;
mod paging;

use region_frame_allocator::RegionFrameAllocator;
pub use self::paging::{PhysAddr, VirtAddr, Mapper};

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame {
    number: usize,
}

impl PhysFrame {
    fn containing_address(address: usize) -> Self {
        Self { number: address / PAGE_SIZE }
    }
    fn start_address(&self) -> PhysAddr {
        self.number * PAGE_SIZE
    }
    fn clone(&self) -> Self {
        Self { number: self.number }
    }
}

impl PhysFrame {
    fn range_inclusive(start: PhysFrame, end: PhysFrame) -> FrameIter {
        FrameIter { start, end }
    }
}

struct FrameIter {
    start: PhysFrame,
    end: PhysFrame,
}

impl Iterator for FrameIter {
    type Item = PhysFrame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
 }


pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame>;
    fn deallocate_frame(&mut self, frame: PhysFrame);
}

pub fn init(multiboot_info: &MultibootInfo) {
    debugprintln!("\nBootloader left us the following memory areas:");
    let memory_map = multiboot_info.memory_map();
    for region in memory_map.iter() {
        debugprintln!("    start: 0x{:0x}, length: {:}", region.base_addr, region.length);
    }

    let kernel_start = multiboot_info.kernel_start();
    let kernel_end = multiboot_info.kernel_end();
    let multiboot_start = multiboot_info.multiboot_start();
    let multiboot_end = multiboot_info.multiboot_end();

    debugprintln!("\nStart of kernel: 0x{:x}", kernel_start);
    debugprintln!("End of kernel: 0x{:x}", kernel_end);

    let mut allocator = RegionFrameAllocator::new(kernel_start, kernel_end, multiboot_start, multiboot_end, memory_map);
    
    unsafe {
        enable_nxe_bit();
    }
    debugprintln!("\nIdentity mapping kernel sections:");
    remap_kernel(&mut allocator, multiboot_info.elf_sections(), multiboot_start, multiboot_end);
    unsafe {
        enable_write_protect_bit();
    }
}