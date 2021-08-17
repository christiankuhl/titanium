use crate::{debugprintln, multiboot::MultibootInfo};

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame {
    number: usize,
}

impl PhysFrame {
    fn containing_address(address: usize) -> Self {
        Self { number: address / PAGE_SIZE }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame>;
    fn deallocate_frame(&mut self, frame: PhysFrame);
}

// pub struct BootInfoFrameAllocator {
//     memory_map: &'static MemoryMap,
//     next: usize,
// }

// impl BootInfoFrameAllocator {
//     pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
//         Self {
//             memory_map,
//             next: 0,
//         }
//     }
//     fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
//         let regions = self.memory_map.iter();
//         let usable_regions = regions
//             .filter(|r| r.usable());
//         let addr_ranges = usable_regions
//             .map(|r| r.base_addr..(r.base_addr + r.length));
//         let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
//         frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr.try_into().unwrap())))
//     }
// }

// unsafe impl FrameAllocator for BootInfoFrameAllocator {
//     fn allocate_frame(&mut self) -> Option<PhysFrame> {
//         let frame = self.usable_frames().nth(self.next);
//         self.next += 1;
//         frame
//     }
// }

// pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
//     let level_4_table = active_level_4_table(physical_memory_offset);
//     OffsetPageTable::new(level_4_table, physical_memory_offset)
// }

// unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {

//     let (level_4_table_frame, _) = Cr3::read();

//     let phys = level_4_table_frame.start_address();
//     let virt = physical_memory_offset + phys.as_u64();
//     let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

//     &mut *page_table_ptr // unsafe
// }


pub fn init(multiboot_info: &MultibootInfo) {
    debugprintln!("\nBootloader left us the following memory areas:");
    for region in multiboot_info.memory_map().iter() {
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

    debugprintln!("\nStart of kernel: 0x{:x}", multiboot_info.kernel_start());
    debugprintln!("End of kernel: 0x{:x}", multiboot_info.kernel_end());
    debugprintln!("Start of multiboot info section: 0x{:x}", multiboot_info.multiboot_start());
    debugprintln!("End of multiboot info section: 0x{:x}", multiboot_info.multiboot_end());
}