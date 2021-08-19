use lazy_static::lazy_static;

use crate::asm::{load_global_descriptor_table, load_task_state_segment, set_code_segment_selector};

use super::{SegmentSelector, idt::{PrivilegeLevel, DescriptorTablePointer}};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const KERNEL_CODE64: u64 = 0x8093000000ffff;
const DPL_RING_3: u64 = 3 << 45;
const PRESENT: u64 = 1 << 47;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { &STACK as *const _ as usize };
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::UserSegment(KERNEL_CODE64));
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    GDT.0.load();
    unsafe {
        set_code_segment_selector(GDT.1.code_selector);
        load_task_state_segment(GDT.1.tss_selector);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct TaskStateSegment {
    reserved_1: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    pub privilege_stack_table: [usize; 3],
    reserved_2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    pub interrupt_stack_table: [usize; 7],
    reserved_3: u64,
    reserved_4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base.
    pub iomap_base: u16,
}

impl TaskStateSegment {
    /// Creates a new TSS with zeroed privilege and interrupt stack table and a zero
    /// `iomap_base`.
    #[inline]
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            privilege_stack_table: [0; 3],
            interrupt_stack_table: [0; 7],
            iomap_base: 0,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable {
    table: [u64; 8],
    next_free: usize,
}

impl GlobalDescriptorTable {
    /// Creates an empty GDT.
    #[inline]
    pub const fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            next_free: 1,
        }
    }
    #[inline]
    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            }
        };

        let rpl = match entry {
            Descriptor::UserSegment(value) => {
                if value & DPL_RING_3 > 0
                {
                    PrivilegeLevel::Ring3
                } else {
                    PrivilegeLevel::Ring0
                }
            }
            Descriptor::SystemSegment(_, _) => PrivilegeLevel::Ring0,
        };

        SegmentSelector::new(index as u16, rpl)
    }

    #[inline]
    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            index
        } else {
            panic!("GDT full");
        }
    }

    #[inline]
    pub fn load(&'static self) {
        unsafe { 
            let ptr = DescriptorTablePointer {
                base: self.table.as_ptr() as u64,
                limit: (self.next_free * 8 - 1) as u16,
            };
            load_global_descriptor_table(&ptr);
        };
    }
}

#[derive(Debug, Clone)]
enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    #[inline]
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        use core::mem::size_of;

        let ptr = tss as *const _ as u64;

        let mut low = PRESENT;
        // base
        low = (!0xffffff0000 & low) | ((ptr & 0xffffff) << 16);
        low = (!0xff00000000000000 & low) | ((ptr & 0xff000000) << 32);
        // limit (the `-1` in needed since the bound is inclusive)
        low = (!0xffff & low) | ((size_of::<TaskStateSegment>() - 1) as u64) & 0xffff;
        // type (0b1001 = available 64-bit tss)
        low = (!0xf0000000000 & low) | (0b1001 << 40);
        let high = (ptr & 0xffffffff00000000) >> 32;

        Descriptor::SystemSegment(low, high)
    }
}

// bitflags! {
//     /// Flags for a GDT descriptor. Not all flags are valid for all descriptor types.
//     pub struct DescriptorFlags: u64 {
//         /// Set by the processor if this segment has been accessed. Only cleared by software.
//         /// _Setting_ this bit in software prevents GDT writes on first use.
//         const ACCESSED          = 1 << 40;
//         /// For 32-bit data segments, sets the segment as writable. For 32-bit code segments,
//         /// sets the segment as _readable_. In 64-bit mode, ignored for all segments.
//         const WRITABLE          = 1 << 41;
//         /// For code segments, sets the segment as “conforming”, influencing the
//         /// privilege checks that occur on control transfers. For 32-bit data segments,
//         /// sets the segment as "expand down". In 64-bit mode, ignored for data segments.
//         const CONFORMING        = 1 << 42;
//         /// This flag must be set for code segments and unset for data segments.
//         const EXECUTABLE        = 1 << 43;
//         /// This flag must be set for user segments (in contrast to system segments).
//         const USER_SEGMENT      = 1 << 44;
//         /// The DPL for this descriptor is Ring 3. In 64-bit mode, ignored for data segments.
//         const DPL_RING_3        = 3 << 45;
//         /// Must be set for any segment, causes a segment not present exception if not set.
//         const PRESENT           = 1 << 47;
//         /// Available for use by the Operating System
//         const AVAILABLE         = 1 << 52;
//         /// Must be set for 64-bit code segments, unset otherwise.
//         const LONG_MODE         = 1 << 53;
//         /// Use 32-bit (as opposed to 16-bit) operands. If [`LONG_MODE`][Self::LONG_MODE] is set,
//         /// this must be unset. In 64-bit mode, ignored for data segments.
//         const DEFAULT_SIZE      = 1 << 54;
//         /// Limit field is scaled by 4096 bytes. In 64-bit mode, ignored for all segments.
//         const GRANULARITY       = 1 << 55;

//         /// Bits `0..=15` of the limit field (ignored in 64-bit mode)
//         const LIMIT_0_15        = 0xFFFF;
//         /// Bits `16..=19` of the limit field (ignored in 64-bit mode)
//         const LIMIT_16_19       = 0xF << 48;
//         /// Bits `0..=23` of the base field (ignored in 64-bit mode, except for fs and gs)
//         const BASE_0_23         = 0xFF_FFFF << 16;
//         /// Bits `24..=31` of the base field (ignored in 64-bit mode, except for fs and gs)
//         const BASE_24_31        = 0xFF << 56;
//     }
// }