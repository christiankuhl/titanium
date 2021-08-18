use lazy_static::lazy_static;
// use volatile::Volatile;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::instructions::port::Port;
use x86_64::registers::control::Cr2;
use x86_64::instructions::interrupts::without_interrupts;
use pic8259::ChainedPics;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin;
// use crate::multitasking::CPUState;
use crate::{
    debugprintln,
    println, print, gdt, idle, 
    drivers::mouse::{
        Mouse, init_mouse, MouseEvent
    },
    shell::vga_buffer::WRITER,
    // multitasking::TASKMANAGER,
};

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Mouse = PIC_2_OFFSET + 4,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_by_zero_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse.as_usize()]
            .set_handler_fn(mouse_interrupt_handler);
        idt
    };
}

extern "x86-interrupt" fn divide_by_zero_handler(s: InterruptStackFrame) {
    println!("EXCEPTION: DIVIDE BY ZERO");
    loop {}
}

lazy_static! {
    static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        spin::Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
            HandleControl::Ignore)
        );
}

lazy_static! {
    static ref MOUSE: spin::Mutex<Mouse> = spin::Mutex::new(Mouse::new());
}

fn init_idt() {
    IDT.load();
    init_mouse();
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(mut stack_frame: InterruptStackFrame) {
    unsafe {
        // let mut taskmgr = TASKMANAGER.lock();
        // let new_stack = taskmgr.switch_task(CPUState::from_stack_frame(&stack_frame));
        // stack_frame.as_mut().extract_inner().instruction_pointer = new_stack.instruction_pointer;
        // stack_frame.as_mut().extract_inner().code_segment = new_stack.code_segment;
        // stack_frame.as_mut().extract_inner().cpu_flags = new_stack.cpu_flags;
        // stack_frame.as_mut().extract_inner().stack_pointer = new_stack.stack_pointer;
        // stack_frame.as_mut().extract_inner().stack_segment = new_stack.stack_segment;
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
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
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

static mut MOUSE_X: i8 = 40;
static mut MOUSE_Y: i8 = 12;

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut mouse = MOUSE.lock();
    let mut port = Port::new(0x60);
    let data: u8 = unsafe { port.read() };
    mouse.add_byte(data as i8);
    if let Some(event) = mouse.event() {
        match event {
            MouseEvent::Move(dx, dy) => {
                without_interrupts(|| {
                    unsafe { move_mouse_cursor(dx, dy) }
                })
            }
            _ => {}
        }
    }
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
}

unsafe fn move_mouse_cursor(dx: i8, dy: i8) {
    let mut screen = WRITER.lock();
    screen.invert(MOUSE_Y as usize, MOUSE_X as usize);
    MOUSE_X += dx / 2;
    if MOUSE_X < 0 { MOUSE_X = 0 };
    if MOUSE_X >= 80 { MOUSE_X = 79 };
    MOUSE_Y -= dy / 2;
    if MOUSE_Y < 0 { MOUSE_Y = 0 };
    if MOUSE_Y >= 25 { MOUSE_Y = 24 };
    screen.invert(MOUSE_Y as usize, MOUSE_X as usize);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    idle();
}

// #[test_case]
// fn test_breakpoint_exception() {
//     // invoke a breakpoint exception
//     x86_64::instructions::interrupts::int3();
// }


pub fn init() {
    debugprintln!("\nInitialising interrupt descriptor table...");
    init_idt();
    
    debugprintln!("\nInitialising interrupt controller...");
    unsafe { PICS.lock().initialize() }; 
}