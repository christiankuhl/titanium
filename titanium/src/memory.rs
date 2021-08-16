use core::convert::TryInto;
use core::ops::Deref;
use core::{str, slice};

// use bootloader::boot_info::{MemoryRegions, MemoryRegionKind};
use x86_64::{
    structures::paging::{
        PhysFrame,
        Size4KiB,
        FrameAllocator,
        PageTable,
        // OffsetPageTable,
    },
    registers::control::Cr3,
    VirtAddr,
    PhysAddr,
};
use crate::println;


#[repr(C)]
struct Tag {
    tag_type: u32,
    size: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct MemoryMapHeaderTag {
    tag_type: u32,
    size: u32,
    entry_size: u32,
    entry_version: u32,
}

pub struct MemoryMap {
    header: *const MemoryMapHeaderTag,
}

pub struct MemoryMapIter {
    header: *const MemoryMapHeaderTag,
    current: *const MemoryRegion,
}

impl MemoryMap {
    fn new(addr: usize) -> Self {
        Self { header: addr as *const MemoryMapHeaderTag }
    }
    pub fn iter(&self) -> MemoryMapIter {
        MemoryMapIter { header: self.header, current: (self.header as usize + 16) as *const MemoryRegion }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MemoryRegion {
    pub base_addr: usize,
    pub length: usize,
    pub region_type: u32,
    _reserved: u32,
}

impl MemoryRegion {
    fn usable(&self) -> bool {
        self.region_type == 1
    }
}

impl Iterator for MemoryMapIter {
    type Item = MemoryRegion; 
    fn next(&mut self) -> Option<MemoryRegion> { 
        let header = unsafe { *self.header };
        if self.current as usize + header.entry_size as usize <= self.header as usize + header.size as usize {
            unsafe { self.current = self.current.offset(1); 
                return Some(*self.current)
            }
        } 
        None
    }
}


// pub fn init(multiboot_ptr: usize) {
//     let addr = find_tag(multiboot_ptr, 6).expect("Bootloader provided no memory map!");
//     let memory_map = MemoryMap::new(addr);
//     for region in memory_map.iter() {
//         println!("{:?}", region);
//     }
// }

pub struct MultibootInfo {
    total_size: u32,
    _reserved: u32,
}

impl MultibootInfo {
    pub fn memory_map(&self) -> MemoryMap {
        let addr = self.find_tag(6).expect("Bootloader provided no memory map!");
        MemoryMap::new(addr)
    }
    pub fn elf_sections(&self) -> ElfSections {
        let addr = self.find_tag(9).expect("Bootloader provided no ELF section headers!");
        ElfSections::new(addr)
    }
    pub fn kernel_start(&self) -> usize {
        self.elf_sections().map(|s| s.addr).min().unwrap().try_into().unwrap()
    }
    pub fn kernel_end(&self) -> usize {
        self.elf_sections().map(|s| s.addr + s.size).max().unwrap().try_into().unwrap()
    }
    pub fn multiboot_start(&self) -> usize {
        self.ptr()
    }
    pub fn multiboot_end(&self) -> usize {
        self.ptr() + self.total_size as usize
    }
    pub fn ptr(&self) -> usize {
        self as *const Self as usize
    }
    fn find_tag(&self, tag_type: u32) -> Option<usize> {
        let mut addr = self.ptr() + 8;
        while addr < self.ptr() + self.total_size as usize {
            let tag = unsafe { &*(addr as *const Tag) };
            if tag.tag_type == tag_type {
                return Some(addr)
            }
            addr += ((tag.size + 7) & !7) as usize;
        }
        None
    }
}

pub struct ElfSections {
    header: *const ElfSectionTagHeader,
    current: *const ElfSectionHeaderRaw,
    remaining_sections: u32,
    offset: *const u8,
}

impl ElfSections {
    fn new(addr: usize) -> Self {
        let header = addr as *const ElfSectionTagHeader;
        let remaining_sections = unsafe { (*header).num_headers };
        let current = unsafe { header.offset(1) as *const ElfSectionHeaderRaw };
        Self { header, current, remaining_sections, offset: current as *const u8 }
    }
    pub fn num_sections(&self) -> u32 {
        unsafe { (*self.header).num_headers }
    }
}

impl Iterator for ElfSections {
    type Item = ElfSectionHeader;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let header = *self.header;
            let string_section_ptr = self.offset.offset((header.string_table_index * header.entry_size) as isize) as *const ElfSectionHeaderRaw;
            let string_ptr = (*string_section_ptr).addr as *const u8;
            if self.remaining_sections > 1 {
                self.remaining_sections -= 1;
                self.current = self.current.offset(1); 
                if (*self.current).section_type != 0 {
                    return Some(ElfSectionHeader::new(*self.current, string_ptr))
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct ElfSectionTagHeader {
    tag_type: u32,
    size: u32,
    num_headers: u32,           
    entry_size: u32,
    string_table_index: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct ElfSectionHeaderRaw {
    pub name_index: u32,
    pub section_type: u32,
    pub flags: u64,
    pub addr: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub addralign: u64,
    pub entry_size: u64,
}

pub struct ElfSectionHeader {
    raw: ElfSectionHeaderRaw,
    string_ptr: *const u8
}

impl ElfSectionHeader {
    fn new(raw: ElfSectionHeaderRaw, string_ptr: *const u8) -> Self {
        Self { raw, string_ptr }
    }
}

impl Deref for ElfSectionHeader {
    type Target = ElfSectionHeaderRaw;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl ElfSectionHeader {
    pub fn name(&self) -> &str {
        if self.name_index == 0 { return "" }
        let mut name_ptr = unsafe { self.string_ptr.offset(self.name_index as isize) };
        // println!("{:x}", name_ptr as usize);
        let strlen = {
            let mut len = 0;
            while unsafe { *name_ptr.offset(len) } != 0 {
                len += 1
            }
            len as usize
        };
        str::from_utf8(unsafe { slice::from_raw_parts(name_ptr, strlen) }).unwrap()
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.usable());
        let addr_ranges = usable_regions
            .map(|r| r.base_addr..(r.base_addr + r.length));
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr.try_into().unwrap())))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

// pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
//     let level_4_table = active_level_4_table(physical_memory_offset);
//     OffsetPageTable::new(level_4_table, physical_memory_offset)
// }

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}
