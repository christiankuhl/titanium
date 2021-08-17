use crate::{debugprintln, multiboot::MultibootInfo};

mod region_frame_allocator;
mod paging;

use region_frame_allocator::RegionFrameAllocator;
pub use self::paging::{PhysAddr, VirtAddr};

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
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame>;
    fn deallocate_frame(&mut self, frame: PhysFrame);
}

pub fn init(multiboot_info: &MultibootInfo) {
    debugprintln!("\nBootloader left us the following memory areas:");
    let memory_map = multiboot_info.memory_map();
    for region in memory_map.iter() {
        debugprintln!("start: 0x{:0x}, length: {:}", region.base_addr, region.length);
    }

    debugprintln!("\nKernel sections:");
    for (idx, section) in multiboot_info.elf_sections().enumerate() {
        let mut name = section.name();
        if name.len() > 30 {
            name = &name[..30];
        }
        debugprintln!("    [{}] {} addr: 0x{:0x}, size: {:0x}, flags: 0x{:0x}", idx, name, section.addr, section.size, section.flags);
    }

    let kernel_start = multiboot_info.kernel_start();
    let kernel_end = multiboot_info.kernel_end();
    let multiboot_start = multiboot_info.multiboot_start();
    let multiboot_end = multiboot_info.multiboot_end();

    debugprintln!("\nStart of kernel: 0x{:x}", kernel_start);
    debugprintln!("End of kernel: 0x{:x}", kernel_end);
    debugprintln!("Start of multiboot info section: 0x{:x}", multiboot_start);
    debugprintln!("End of multiboot info section: 0x{:x}", multiboot_end);

    let mut allocator = RegionFrameAllocator::new(kernel_start, kernel_end, multiboot_start, multiboot_end, memory_map);
    paging::test_paging(&mut allocator);
}