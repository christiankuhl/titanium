use core::ops::Deref;
use core::{str, slice};

pub struct ElfSections {
    header: *const ElfSectionTagHeader,
    current: *const ElfSectionHeader,
    remaining_sections: u32,
    offset: *const u8,
}

impl ElfSections {
    pub fn new(addr: usize) -> Self {
        let header = addr as *const ElfSectionTagHeader;
        let remaining_sections = unsafe { (*header).num_headers };
        let current = unsafe { header.offset(1) as *const ElfSectionHeader };
        Self { header, current, remaining_sections, offset: current as *const u8 }
    }
    pub fn num_sections(&self) -> u32 {
        unsafe { (*self.header).num_headers }
    }
}

impl Iterator for ElfSections {
    type Item = ElfSection;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let header = *self.header;
            let string_section_ptr = self.offset.offset((header.string_table_index * header.entry_size) as isize) as *const ElfSectionHeader;
            let string_ptr = (*string_section_ptr).addr as *const u8;
            if self.remaining_sections > 1 {
                self.remaining_sections -= 1;
                self.current = self.current.offset(1); 
                if (*self.current).section_type != 0 {
                    return Some(ElfSection::new(*self.current, string_ptr))
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
pub struct ElfSectionHeader {
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

pub struct ElfSection {
    header: ElfSectionHeader,
    string_ptr: *const u8
}

impl ElfSection {
    fn new(header: ElfSectionHeader, string_ptr: *const u8) -> Self {
        Self { header: header, string_ptr }
    }
}

impl Deref for ElfSection {
    type Target = ElfSectionHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

impl ElfSection {
    pub fn name(&self) -> &str {
        if self.name_index == 0 { return "" }
        let mut name_ptr = unsafe { self.string_ptr.offset(self.name_index as isize) };
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