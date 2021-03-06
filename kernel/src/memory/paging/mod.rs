use core::ops::{Deref, DerefMut};
use elf::ElfSections;

use super::{FrameAllocator, PhysFrame, PAGE_SIZE};
use crate::{
    asm::{tlb_flush_all, Cr3},
    log,
};

mod entry;
mod mapper;
mod table;
mod temporary_page;

pub use self::entry::*;
pub use self::mapper::Mapper;
use self::table::Table;
use self::temporary_page::TemporaryPage;
use super::REGION_FRAME_ALLOCATOR;

const ENTRY_COUNT: usize = 512;

pub type PhysAddr = usize;
pub type VirtAddr = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing_address(address: VirtAddr) -> Self {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000, "invalid address: 0x{:x}", address);
        Self { number: address / PAGE_SIZE }
    }
    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }
    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }
    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }
    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }
    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter { start, end }
    }
}

pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> Self {
        Self { mapper: Mapper::new() }
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut temporary_page::TemporaryPage, f: F)
    where
        F: FnOnce(&mut Mapper),
    {
        {
            let backup = PhysFrame::containing_address(Cr3::p4_address());
            // map temporary_page to current p4 table
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);
            // overwrite recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb_flush_all();
            // execute f in the new context
            f(self);
            // restore recursive mapping to original p4 table
            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb_flush_all();
        }
        temporary_page.unmap(self);
    }
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable { p4_frame: PhysFrame::containing_address(Cr3::p4_address()) };
        unsafe {
            Cr3::write(new_table.p4_frame.start_address());
        }
        old_table
    }
}

pub struct InactivePageTable {
    p4_frame: PhysFrame,
}

impl InactivePageTable {
    pub fn new(frame: PhysFrame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            // set up recursive mapping for the table
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);
        InactivePageTable { p4_frame: frame }
    }
}

pub fn remap_kernel<A>(
    allocator: &mut spin::MutexGuard<A>,
    elf_sections: ElfSections,
    multiboot_start: usize,
    multiboot_end: usize,
    shstrtab_start: usize,
    shstrtab_end: usize,
) -> ActivePageTable
where
    A: FrameAllocator,
{
    let mut temporary_page = TemporaryPage::new(Page { number: 0xdeadbeef }, allocator.deref_mut());

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("No more frames available!");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        log!("\nIdentity mapping VGA text buffer at 0xb8000...");
        let vga_buffer_frame = PhysFrame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, allocator.deref_mut());
        log!("\nIdentity mapping multiboot info section...");
        log!("    data start: {:#x}", multiboot_start);
        log!("    data end: {:#x}", multiboot_end);
        log!("    .shstrtab start: {:#x}", shstrtab_start);
        log!("    .shstrtab end: {:#x}", shstrtab_end);
        for frame in PhysFrame::range_inclusive(
            PhysFrame::containing_address(multiboot_start),
            PhysFrame::containing_address(shstrtab_end - 1),
        ) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator.deref_mut());
        }
        log!("\nIdentity mapping kernel sections...");
        for (idx, section) in elf_sections.enumerate() {
            if !section.is_allocated() {
                continue;
            }
            assert!(section.addr % (PAGE_SIZE as u64) == 0, "Sections need to be page aligned!");
            let mut name = section.name();
            if name.len() > 30 {
                name = &name[..30];
            }
            log!(
                "    [{}] {} addr: 0x{:0x}, size: {:0x}, flags: 0x{:0x}",
                idx,
                name,
                section.addr,
                section.size,
                section.flags
            );
            let flags = EntryFlags::from_elf_section(&section);
            let start_frame = PhysFrame::containing_address(section.addr as usize);
            let end_frame = PhysFrame::containing_address((section.addr + section.size - 1) as usize);
            for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator.deref_mut());
            }
        }
    });
    let old_table = active_table.switch(new_table);
    // turn the old p4 page into a guard page
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator.deref_mut());
    log!("\nKernel stack guard page is at {:#x}...", old_p4_page.start_address());
    active_table
}

pub fn allocate_anywhere(virtaddr: VirtAddr, size: usize, flags: EntryFlags) -> VirtAddr {
    let mut active_page_table = unsafe { ActivePageTable::new() };
    let mut allocator = REGION_FRAME_ALLOCATOR.lock();
    let page_range = Page::range_inclusive(Page::containing_address(virtaddr), Page::containing_address(virtaddr + size - 1));
    for page in page_range {
        if active_page_table.translate_page(page) == None {
            let frame = allocator.allocate_frame().unwrap();
            active_page_table.map_to(page, frame, flags, allocator.deref_mut());
        }
    }
    Page::containing_address(virtaddr).start_address()
}

pub fn allocate_identity_mapped(addr: PhysAddr, size: usize, flags: EntryFlags) -> VirtAddr {
    let mut active_page_table = unsafe { ActivePageTable::new() };
    let mut allocator = REGION_FRAME_ALLOCATOR.lock();
    let range = PhysFrame::range_inclusive(PhysFrame::containing_address(addr), PhysFrame::containing_address(addr + size - 1));
    for frame in range {
        if active_page_table.translate_page(Page::containing_address(frame.start_address())) == None {
            active_page_table.identity_map(frame, flags, allocator.deref_mut());
        }
    }
    Page::containing_address(addr).start_address()
}

const PROTECTION_VIOLATION: u64 = 1 << 0;
const CAUSED_BY_WRITE: u64 = 1 << 1;
const USER_MODE: u64 = 1 << 2;
const MALFORMED_TABLE: u64 = 1 << 3;
const INSTRUCTION_FETCH: u64 = 1 << 4;

pub struct PageFaultErrorCode;

impl PageFaultErrorCode {
    pub fn from(error_code: u64) -> &'static str {
        if error_code & PROTECTION_VIOLATION > 0 {
            return "PROTECTION VIOLATION";
        } else if error_code & CAUSED_BY_WRITE > 0 {
            return "CAUSED_BY_WRITE";
        } else if error_code & USER_MODE > 0 {
            return "USER_MODE";
        } else if error_code & MALFORMED_TABLE > 0 {
            return "MALFORMED_TABLE";
        } else if error_code & INSTRUCTION_FETCH > 0 {
            return "INSTRUCTION_FETCH";
        } else {
            return "UNSPECIFIED";
        }
    }
}

pub trait Translate {
    fn translate(&self) -> PhysAddr;
}

impl Translate for VirtAddr {
    fn translate(&self) -> PhysAddr {
        let active_page_table = unsafe { ActivePageTable::new() };
        active_page_table.translate(*self).unwrap()
    }
}
