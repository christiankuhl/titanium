use lazy_static::lazy_static;

use crate::asm::{load_global_descriptor_table, load_task_state_segment, set_code_segment_selector};

use super::{
    idt::{DescriptorTablePointer, PrivilegeLevel},
    SegmentSelector,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const KERNEL_CODE64: u64 = 0xa0_9800_0000_ffff;
const KERNEL_DATA64: u64 = 0xc0_9200_0000_ffff;
const USER_CODE64: u64 = 0xa0_f800_0000_ffff;
const USER_DATA64: u64 = 0xc0_f200_0000_ffff;
const DPL_RING_3: u64 = 3 << 45;
const PRESENT: u64 = 1 << 47;

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut DOUBLE_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = unsafe { &DOUBLE_FAULT_STACK as *const _ as usize };
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut PRIVILEGE_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = unsafe { &PRIVILEGE_STACK as *const _ as usize };
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_selector = gdt.add_entry(Descriptor::UserSegment(KERNEL_CODE64));
        let kernel_data_selector = gdt.add_entry(Descriptor::UserSegment(KERNEL_DATA64));
        gdt.add_entry(Descriptor::UserSegment(0));
        let user_data_selector = gdt.add_entry(Descriptor::UserSegment(USER_DATA64));
        let user_code_selector = gdt.add_entry(Descriptor::UserSegment(USER_CODE64));
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { kernel_code_selector, user_code_selector, tss_selector })
    };
}

struct Selectors {
    kernel_code_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    GDT.0.load();
    unsafe {
        set_code_segment_selector(GDT.1.kernel_code_selector);
        load_task_state_segment(GDT.1.tss_selector);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TaskStateSegment {
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
        GlobalDescriptorTable { table: [0; 8], next_free: 1 }
    }
    #[inline]
    fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
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
                if value & DPL_RING_3 > 0 {
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
            let ptr = DescriptorTablePointer { base: self.table.as_ptr() as u64, limit: (self.next_free * 8 - 1) as u16 };
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
        low = (!0xff_ffff_0000 & low) | ((ptr & 0xff_ffff) << 16);
        low = (!0xff00_0000_0000_0000 & low) | ((ptr & 0xff00_0000) << 32);
        // limit (the `-1` is needed since the bound is inclusive)
        low = (!0xffff & low) | ((size_of::<TaskStateSegment>() - 1) as u64) & 0xffff;
        // type (0b1001 = available 64-bit tss)
        low = (!0xf00_0000_0000 & low) | (0b1001 << 40);
        let high = (ptr & 0xffffffff00000000) >> 32;

        Descriptor::SystemSegment(low, high)
    }
}
