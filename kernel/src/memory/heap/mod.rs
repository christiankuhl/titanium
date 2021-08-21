use crate::debugprintln;

use self::fixed_size_block::FixedSizeBlockAllocator;
use super::{
    paging::{ActivePageTable, EntryFlags, Flags, Page},
    region_frame_allocator::RegionFrameAllocator,
};

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub mod fixed_size_block;
pub mod linked_list;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked { inner: spin::Mutex::new(inner) }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

pub fn init(active_table: &mut ActivePageTable, frame_allocator: &mut RegionFrameAllocator) {
    debugprintln!("\nInitialising heap...");
    debugprintln!("    Heap start: {:#x}", HEAP_START);
    debugprintln!("    Heap end: {:#x}", HEAP_START + HEAP_SIZE - 1);
    let page_range = {
        let heap_start_page = Page::containing_address(HEAP_START);
        let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        active_table.map(page, EntryFlags::WRITABLE, frame_allocator);
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
}
