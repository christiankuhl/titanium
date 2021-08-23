use core::mem::size_of;

use crate::asm::{code_segment_selector, load_interrupt_descriptor_table};
use crate::drivers::pic::{PIC_1_OFFSET, PIC_2_OFFSET};

pub type HandlerFunc = extern "C" fn() -> !;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
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

pub struct InterruptDescriptorTable([IDTEntry; 45]);

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self([IDTEntry::missing(); 45])
    }
    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) -> &mut EntryOptions {
        self.0[entry as usize] = IDTEntry::new(code_segment_selector(), handler);
        &mut self.0[entry as usize].options
    }
    pub fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };
        unsafe {
            load_interrupt_descriptor_table(&ptr);
        }
    }
}

#[repr(C, packed)]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: u64,
}

#[repr(u8)]
pub enum Interrupt {
    DivideError = 0x0,
    Debug = 0x1,
    NonMaskableInterrupt = 0x2,
    Breakpoint = 0x3,
    Overflow = 0x4,
    BoundRangeExceeded = 0x5,
    InvalidOpcode = 0x6,
    DeviceNotAvailable = 0x7,
    DoubleFault = 0x8,
    InvalidTSS = 0xa,
    SegmentNotPresent = 0xb,
    StackSegmentFault = 0xc,
    GeneralProtectionFault = 0xd,
    PageFault = 0xe,
    AlignmentCheck = 0x11,
    SIMDException = 0x13,
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
    Mouse = PIC_2_OFFSET + 4,
}
