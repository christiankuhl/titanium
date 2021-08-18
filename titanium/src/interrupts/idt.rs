use core::mem::size_of;

use crate::{
    asm_wrappers::{code_segment_selector, load_interrupt_descriptor_table},
    println,
};

pub type HandlerFunc = extern "C" fn() -> !;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IDTEntry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl IDTEntry {
    fn new(gdt_selector: SegmentSelector, handler: HandlerFunc) -> Self {
        let pointer = handler as u64;
        Self {
            gdt_selector,
            pointer_low: pointer as u16,
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            options: EntryOptions::new(),
            reserved: 0,
        }
    }
    fn missing() -> Self {
        Self {
            gdt_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

impl EntryOptions {
    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }
    fn minimal() -> Self {
        Self(0xe00)
    }
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0 = (0x7fff & self.0) | ((present as u16) << 15);
        self
    }
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0 = (0xfeff & self.0) | ((!disable as u16) << 8);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0 = (0x9fff & self.0) | ((dpl & 0x3) << 13);
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0 = (0xfff8 & self.0) | (index & 0x7);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    #[inline]
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }
    #[inline]
    pub fn index(self) -> u16 {
        self.0 >> 3
    }
    #[inline]
    pub fn rpl(self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0 & 0x3)
    }
    #[inline]
    pub fn set_rpl(&mut self, rpl: PrivilegeLevel) {
        self.0 = (!0x3 & self.0) | rpl as u16;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl PrivilegeLevel {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            i => panic!("{} is not a valid privilege level", i),
        }
    }
}

pub struct InterruptDescriptorTable([IDTEntry; 16]);

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self([IDTEntry::missing(); 16])
    }
    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) -> &mut EntryOptions {
        self.0[entry as usize] = IDTEntry::new(code_segment_selector(), handler);
        &mut self.0[entry as usize].options
    }
    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer { base: self as *const _ as u64, limit: (size_of::<Self>() - 1) as u16 };
        println!("{:?}", &ptr);
        unsafe {
            load_interrupt_descriptor_table(&ptr);
        };
    }
}

#[derive(Debug)]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: u64,
}
