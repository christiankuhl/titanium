use super::{FrameAllocator, PhysFrame, PAGE_SIZE};
use crate::multiboot::memory_map::*;

pub struct RegionFrameAllocator {
    next_free_frame: PhysFrame,
    current_region: Option<&'static MemoryRegion>,
    regions: MemoryMapIter,
    kernel_start: PhysFrame,
    kernel_end: PhysFrame,
    multiboot_start: PhysFrame,
    multiboot_end: PhysFrame,
    shstrtab_start: PhysFrame,
    shstrtab_end: PhysFrame,
}

impl RegionFrameAllocator {
    pub fn new(
        kernel_start: usize,
        kernel_end: usize,
        multiboot_start: usize,
        multiboot_end: usize,
        shstrtab_start: usize,
        shstrtab_end: usize,
        memory_map: MemoryMap,
    ) -> Self {
        let mut allocator = Self {
            next_free_frame: PhysFrame::containing_address(0),
            current_region: None,
            regions: memory_map.iter(),
            kernel_start: PhysFrame::containing_address(kernel_start),
            kernel_end: PhysFrame::containing_address(kernel_end),
            multiboot_start: PhysFrame::containing_address(multiboot_start),
            multiboot_end: PhysFrame::containing_address(multiboot_end),
            shstrtab_start: PhysFrame::containing_address(shstrtab_start),
            shstrtab_end: PhysFrame::containing_address(shstrtab_end),
        };
        allocator.choose_next_region();
        allocator
    }

    fn choose_next_region(&mut self) {
        self.current_region = self
            .regions
            .clone()
            .filter(|region| region.usable())
            .filter(|region| region.length > PAGE_SIZE)
            .filter(|region| {
                let address = region.base_addr + region.length - 1;
                PhysFrame::containing_address(address as usize) >= self.next_free_frame
            })
            .min_by_key(|region| region.base_addr);

        if let Some(region) = self.current_region {
            let start_frame = PhysFrame::containing_address(region.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl FrameAllocator for RegionFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if let Some(region) = self.current_region {
            // "Clone" the frame to return it if it's free. PhysFrame doesn't
            // implement Clone, but we can construct an identical frame.
            let frame = PhysFrame { number: self.next_free_frame.number };

            // the last frame of the current region
            let current_area_last_frame = {
                let address = region.base_addr + region.length - 1;
                PhysFrame::containing_address(address as usize)
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next region
                self.choose_next_region();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // `frame` is used by the kernel
                self.next_free_frame = PhysFrame { number: self.kernel_end.number + 1 };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = PhysFrame { number: self.multiboot_end.number + 1 };
            } else if frame >= self.shstrtab_start && frame <= self.shstrtab_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = PhysFrame { number: self.shstrtab_end.number + 1 };
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame.number += 1;
                return Some(frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frame()
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frame(&mut self, frame: PhysFrame) {
        // todo!("To be done...")
    }
}
