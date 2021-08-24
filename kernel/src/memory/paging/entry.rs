use elf::ElfSection;
use super::PhysFrame;

#[derive(Debug)]
pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags_set(&self, flags: u64) -> bool {
        self.0 & flags > 0
    }

    pub fn pointed_frame(&self) -> Option<PhysFrame> {
        if self.flags_set(EntryFlags::PRESENT) {
            Some(PhysFrame::containing_address(self.0 as usize & 0x000fffff_fffff000))
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: PhysFrame, flags: EntryFlags) {
        self.0 = (frame.start_address() as u64) | flags;
    }
}

pub type EntryFlags = u64;

pub trait Flags {
    const PRESENT: u64;
    const WRITABLE: u64;
    const USER_ACCESSIBLE: u64;
    const WRITE_THROUGH: u64;
    const NO_CACHE: u64;
    const ACCESSED: u64;
    const DIRTY: u64;
    const HUGE: u64;
    const GLOBAL: u64;
    const NO_EXECUTE: u64;
    fn empty() -> Self;
    fn from_elf_section(section: &ElfSection) -> u64;
}

impl Flags for EntryFlags {
    const PRESENT: u64 = 1 << 0;
    const WRITABLE: u64 = 1 << 1;
    const USER_ACCESSIBLE: u64 = 1 << 2;
    const WRITE_THROUGH: u64 = 1 << 3;
    const NO_CACHE: u64 = 1 << 4;
    const ACCESSED: u64 = 1 << 5;
    const DIRTY: u64 = 1 << 6;
    const HUGE: u64 = 1 << 7;
    const GLOBAL: u64 = 1 << 8;
    const NO_EXECUTE: u64 = 1 << 63;
    fn empty() -> Self {
        0
    }
    fn from_elf_section(section: &ElfSection) -> u64 {
        let mut flags = EntryFlags::empty();
        if section.is_allocated() {
            // section is loaded to memory
            flags = flags | EntryFlags::PRESENT;
        }
        if section.is_writable() {
            flags = flags | EntryFlags::WRITABLE;
        }
        if !section.is_executable() {
            flags = flags | EntryFlags::NO_EXECUTE;
        }
        flags
    }
}
