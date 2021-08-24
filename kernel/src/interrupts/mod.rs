use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;

use crate::multitasking::ThreadRegisters;
use crate::{log, print, println};
use crate::asm::{idle, page_fault_linear_address, without_interrupts, inb};
use crate::drivers::pic::PICS;
use crate::drivers::keyboard::KEYBOARD;
use crate::drivers::mouse::{MOUSE, init_mouse, move_mouse_cursor, MouseEvent};

mod idt;
mod gdt;
#[macro_use]
mod asm;

use self::idt::InterruptDescriptorTable;
pub use self::idt::{DescriptorTablePointer, SegmentSelector, Interrupt};

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
        idt
    };
}

#[no_mangle]
extern "C" fn divide_by_zero_handler(stack_frame: &InterruptStackFrame) -> ! {
    println!("EXCEPTION: DIVIDE BY ZERO");
    println!("{:#x?}", stack_frame);
    idle();
}

#[no_mangle]
extern "C" fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) -> ! {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:#x?}", page_fault_linear_address());
    println!("Error Code: {:?}", error_code);
    println!("{:#x?}", stack_frame);
    idle();
}

#[no_mangle]
extern "C" fn breakpoint_handler(stack_frame: &InterruptStackFrame, rsp: u64) -> u64 {
    println!("EXCEPTION: BREAKPOINT\n{:#x?}", stack_frame);
    rsp
}

#[no_mangle]
extern "C" fn double_fault_handler(stack_frame: &InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#x?}", stack_frame);
}

#[no_mangle]
extern "C" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame, rsp: u64) -> u64 {
    unsafe {
        let new_rsp = {
            let mut scheduler = crate::multitasking::SCHEDULER.lock();
            let cpu_state = rsp as *mut ThreadRegisters;
            scheduler.switch_thread(cpu_state) as *const _ as u64
        };
        PICS.lock().notify_end_of_interrupt(Interrupt::Timer as u8);
        new_rsp
    }
}

#[no_mangle]
extern "C" fn keyboard_interrupt_handler(_stack_frame: &InterruptStackFrame, rsp: u64) -> u64 {
    let mut keyboard = KEYBOARD.lock();
    let scancode: u8 = unsafe { inb(0x60) };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
    unsafe {
        PICS.lock().notify_end_of_interrupt(Interrupt::Keyboard as u8);
    }
    rsp
}

#[no_mangle]
extern "C" fn mouse_interrupt_handler(_stack_frame: &InterruptStackFrame, rsp: u64) -> u64 {
    let mut mouse = MOUSE.lock();
    let data: u8 = unsafe { inb(0x60) };
    mouse.add_byte(data as i8);
    if let Some(event) = mouse.event() {
        match event {
            MouseEvent::Move(dx, dy) => without_interrupts(|| unsafe { move_mouse_cursor(dx, dy) }),
            _ => {}
        }
    }
    unsafe {
        PICS.lock().notify_end_of_interrupt(Interrupt::Mouse as u8);
    }
    rsp
}

pub fn init() {
    log!("\nInitialising global descriptor table...");
    gdt::init();
    log!("\nInitialising interrupt descriptor table...");
    init_idt();
    log!("\nInitialising interrupt controller...");
    unsafe { PICS.lock().initialize() };
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

fn init_idt() {
    IDT.load();
    init_mouse();
}






