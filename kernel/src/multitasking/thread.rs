use alloc::boxed::Box;
use core::{fmt::Debug, mem::size_of};

const THREAD_STACK_SIZE: usize = 4096;

#[derive(Debug)]
#[repr(C)]
pub struct ThreadRegisters {
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

#[derive(Debug)]
pub struct Thread<'a> {
    stack: Box<[u8; 4096]>,
    pub registers: &'a ThreadRegisters,
}

impl<'a> Thread<'a> {
    pub fn new(entry_point: fn() -> !) -> Self {
        let stack = Box::new([0; THREAD_STACK_SIZE]);
        let ptr = ((stack.as_ptr() as usize & !0x15) + THREAD_STACK_SIZE - size_of::<ThreadRegisters>()) as *mut ThreadRegisters;
        let mut registers = unsafe { &mut *ptr };
        registers.rflags = 0x202;
        registers.cs = 0x8;
        registers.rip = entry_point as u64;
        registers.rsp = ptr as u64;
        Self { stack, registers }
    }
}
