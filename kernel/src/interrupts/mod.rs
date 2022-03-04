use lazy_static::lazy_static;

use crate::drivers::mouse::init_mouse;
use crate::drivers::pic::PICS;
use crate::log;

mod gdt;
mod idt;
#[macro_use]
mod asm;
mod handlers;

use self::handlers::*;
use self::idt::InterruptDescriptorTable;
pub use self::idt::{DescriptorTablePointer, Interrupt, SegmentSelector};
pub use self::asm::enter_userspace;

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        use Interrupt::*;
        let mut idt = InterruptDescriptorTable::new();
        idt.set_handler(DivideError as u8, handler!(divide_by_zero_handler));
        idt.set_handler(Breakpoint as u8, handler!(breakpoint_handler));
        idt.set_handler(PageFault as u8, handler_with_error_code!(page_fault_handler));
        idt.set_handler(DoubleFault as u8, handler_with_error_code!(double_fault_handler))
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        idt.set_handler(Timer as u8, handler!(timer_interrupt_handler));
        idt.set_handler(Keyboard as u8, handler!(keyboard_interrupt_handler));
        idt.set_handler(Mouse as u8, handler!(mouse_interrupt_handler));
        idt.set_handler(Syscall as u8, handler!(syscall_handler));
        idt.set_handler(AHCI as u8, handler!(ahci_interrupt_handler));
        idt
    };
}

pub fn init() {
    log!("\nInitialising global descriptor table...");
    gdt::init();
    log!("\nInitialising interrupt descriptor table...");
    init_idt();
    log!("\nInitialising interrupt controller...");
    let mut pics = PICS.lock();
    unsafe {
        pics.initialize();
    };
}

fn init_idt() {
    IDT.load();
    init_mouse();
}
