use crate::println;
use lazy_static::lazy_static;

mod idt;

use self::idt::InterruptDescriptorTable;
pub use self::idt::{DescriptorTablePointer, SegmentSelector};

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.set_handler(0, divide_by_zero_handler);
        idt.set_handler(1, divide_by_zero_handler);
        idt.set_handler(2, divide_by_zero_handler);
        idt.set_handler(3, divide_by_zero_handler);
        idt.set_handler(14, page_fault_handler);
        // idt.breakpoint.set_handler_fn(breakpoint_handler);
        // unsafe {
        //     idt.double_fault.set_handler_fn(double_fault_handler)
        //         .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        // }
        // idt.page_fault.set_handler_fn(page_fault_handler);
        // idt[InterruptIndex::Timer.as_usize()]
        //     .set_handler_fn(timer_interrupt_handler);
        // idt[InterruptIndex::Keyboard.as_usize()]
        //     .set_handler_fn(keyboard_interrupt_handler);
        // idt[InterruptIndex::Mouse.as_usize()]
        //     .set_handler_fn(mouse_interrupt_handler);
        idt
    };
}

extern "C" fn divide_by_zero_handler() -> ! {
    println!("EXCEPTION: DIVIDE BY ZERO");
    loop {}
}

extern "C" fn page_fault_handler() -> ! {
    println!("EXCEPTION: PAGE FAULT");
    loop {}
}

pub fn init() {
    IDT.load();
}
