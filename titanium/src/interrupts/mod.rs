use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::instructions::port::Port;

use crate::{println, debugprintln, print, gdt, asm_wrappers::idle};
use crate::drivers::pic::PICS;
use crate::drivers::keyboard::KEYBOARD;
use crate::drivers::mouse::{MOUSE, init_mouse, move_mouse_cursor, MouseEvent};

use x86_64::registers::control::Cr2;

mod idt;
#[macro_use]
mod asm_wrappers;

use self::idt::{InterruptDescriptorTable, Interrupt};
pub use self::idt::{DescriptorTablePointer, SegmentSelector};

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        use Interrupt::*;
        let mut idt = InterruptDescriptorTable::new();
        idt.set_handler(DivideError as u8, handler!(divide_by_zero_handler));
        idt.set_handler(Breakpoint as u8, handler!(breakpoint_handler));
        idt.set_handler(PageFault as u8, handler_with_error_code!(page_fault_handler));
        unsafe {
            idt.set_handler(DoubleFault as u8, handler_with_error_code!(double_fault_handler))
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.set_handler(Timer as u8, handler!(timer_interrupt_handler));
        idt.set_handler(Keyboard as u8, handler!(keyboard_interrupt_handler));
        idt.set_handler(Mouse as u8, handler!(mouse_interrupt_handler));
        idt
    };
}

#[no_mangle]
extern "C" fn divide_by_zero_handler(stack_frame: &InterruptStackFrame) -> ! {
    let x = (1u64, 2u64, 3u64);
    let y = Some(x);
    for i in (0..100).map(|z| (z, z - 1)) {}
    println!("EXCEPTION: DIVIDE BY ZERO");
    println!("{:#?}", stack_frame);
    idle();
}

#[no_mangle]
extern "C" fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    idle();
}

#[no_mangle]
extern "C" fn breakpoint_handler(stack_frame: &InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[no_mangle]
extern "C" fn double_fault_handler(stack_frame: &InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[no_mangle]
extern "C" fn timer_interrupt_handler(mut stack_frame: &InterruptStackFrame) {
    unsafe {
        // let mut taskmgr = TASKMANAGER.lock();
        // let new_stack = taskmgr.switch_task(CPUState::from_stack_frame(&stack_frame));
        // stack_frame.as_mut().extract_inner().instruction_pointer = new_stack.instruction_pointer;
        // stack_frame.as_mut().extract_inner().code_segment = new_stack.code_segment;
        // stack_frame.as_mut().extract_inner().cpu_flags = new_stack.cpu_flags;
        // stack_frame.as_mut().extract_inner().stack_pointer = new_stack.stack_pointer;
        // stack_frame.as_mut().extract_inner().stack_segment = new_stack.stack_segment;
        PICS.lock().notify_end_of_interrupt(Interrupt::Timer as u8);
    }
}

#[no_mangle]
extern "C" fn keyboard_interrupt_handler(_stack_frame: &InterruptStackFrame) {
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
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
}

#[no_mangle]
extern "C" fn mouse_interrupt_handler(_stack_frame: &InterruptStackFrame) {
    let mut mouse = MOUSE.lock();
    let mut port = Port::new(0x60);
    let data: u8 = unsafe { port.read() };
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
}

pub fn init() {
    debugprintln!("\nInitialising interrupt descriptor table...");
    init_idt();

    debugprintln!("\nInitialising interrupt controller...");
    unsafe { PICS.lock().initialize() };
}

#[derive(Debug)]
#[repr(C)]
struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

fn init_idt() {
    IDT.load();
    init_mouse();
}






