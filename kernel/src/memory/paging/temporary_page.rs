use super::{table::Level1, ActivePageTable, EntryFlags, Flags, FrameAllocator, Page, PhysFrame, Table, VirtAddr};

pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    pub fn new<A>(page: Page, allocator: &mut A) -> Self
    where
        A: FrameAllocator,
    {
        Self { page, allocator: TinyAllocator::new(allocator) }
    }
    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page.
    pub fn map(&mut self, frame: PhysFrame, active_table: &mut ActivePageTable) -> VirtAddr {
        assert!(active_table.translate_page(self.page).is_none(), "Temporary page is already mapped");
        active_table.map_to(self.page, frame, EntryFlags::WRITABLE, &mut self.allocator);
        self.page.start_address()
    }

    /// Unmaps the temporary page in the active table.
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator)
    }

    /// Maps the temporary page to the given page table frame in the active
    /// table. Returns a reference to the now mapped table.
    pub fn map_table_frame(&mut self, frame: PhysFrame, active_table: &mut ActivePageTable) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }
}

struct TinyAllocator([Option<PhysFrame>; 3]);

impl FrameAllocator for TinyAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }
    fn deallocate_frame(&mut self, frame: PhysFrame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("TinyAllocator can hold only 3 frames.");
    }
}

impl TinyAllocator {
    fn new<A>(allocator: &mut A) -> TinyAllocator
    where
        A: FrameAllocator,
    {
        let mut f = || allocator.allocate_frame();
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }
}
