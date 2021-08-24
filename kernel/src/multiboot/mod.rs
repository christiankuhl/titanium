use core::convert::TryInto;

pub mod memory_map;

use elf::*;
use memory_map::*;

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
        self.elf_sections().filter(|s| s.is_allocated()).map(|s| s.addr).min().unwrap().try_into().unwrap()
    }
    pub fn kernel_end(&self) -> usize {
        self.elf_sections().filter(|s| s.is_allocated()).map(|s| s.addr + s.size).max().unwrap().try_into().unwrap()
    }
    pub fn multiboot_start(&self) -> usize {
        self.ptr()
    }
    pub fn multiboot_end(&self) -> usize {
        self.ptr() + self.total_size as usize
    }
    pub fn shstrtab_start(&self) -> usize {
        self.elf_sections().map(|s| s.name_ptr as usize).min().unwrap()
    }
    pub fn shstrtab_end(&self) -> usize {
        self.elf_sections().map(|s| s.name_ptr as usize + s.name_len).max().unwrap()
    }
    pub fn ptr(&self) -> usize {
        self as *const Self as usize
    }
    fn find_tag(&self, tag_type: u32) -> Option<usize> {
        let mut addr = self.ptr() + 8;
        while addr < self.ptr() + self.total_size as usize {
            let tag = unsafe { &*(addr as *const Tag) };
            if tag.tag_type == tag_type {
                return Some(addr);
            }
            addr += ((tag.size + 7) & !7) as usize;
        }
        None
    }
}

#[repr(C)]
struct Tag {
    tag_type: u32,
    size: u32,
}
