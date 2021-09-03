use pc_keyboard::DecodedKey;

use crate::asm::{inb, page_fault_linear_address, without_interrupts};
use crate::drivers::ahci::AHCI_CONTROLLERS;
use crate::drivers::keyboard::KEYBOARD;
use crate::drivers::mouse::{move_mouse_cursor, MouseEvent, MOUSE};
use crate::drivers::pic::PICS;
use crate::memory::PageFaultErrorCode;
use crate::multitasking::ThreadRegisters;
use crate::{print, println};

use super::Interrupt;

#[no_mangle]
pub extern "C" fn divide_by_zero_handler(stack_frame: &InterruptStackFrame) -> ! {
    println!("EXCEPTION: DIVIDE BY ZERO");
    panic!("{:#x?}", stack_frame);
}

#[no_mangle]
pub extern "C" fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) -> ! {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:#x?}", page_fault_linear_address());
    println!("Error Code: {}", PageFaultErrorCode::from(error_code));
    panic!("{:#x?}", stack_frame);
}

#[no_mangle]
pub extern "C" fn breakpoint_handler(stack_frame: &InterruptStackFrame, rsp: u64) -> u64 {
    println!("EXCEPTION: BREAKPOINT\n{:#x?}", stack_frame);
    rsp
}

#[no_mangle]
pub extern "C" fn double_fault_handler(stack_frame: &InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#x?}", stack_frame);
}

#[no_mangle]
pub extern "C" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame, rsp: u64) -> u64 {
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
pub extern "C" fn keyboard_interrupt_handler(_stack_frame: &InterruptStackFrame, rsp: u64) -> u64 {
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
pub extern "C" fn mouse_interrupt_handler(_stack_frame: &InterruptStackFrame, rsp: u64) -> u64 {
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

#[no_mangle]
pub extern "C" fn syscall_handler(_stack_frame: &InterruptStackFrame, rsp: u64) -> u64 {
    println!("Received syscall!");
    rsp
}

#[no_mangle]
pub extern "C" fn ahci_interrupt_handler(_stack_frame: &InterruptStackFrame, rsp: u64) -> u64 {
    for ctrl in AHCI_CONTROLLERS.lock().iter_mut() {
        if ctrl.handle_interrupt() {
            unsafe {
                crate::drivers::pic::PICS.lock().notify_end_of_interrupt(ctrl.interrupt_vector());
            }
            break;
        }
    }
    rsp
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
