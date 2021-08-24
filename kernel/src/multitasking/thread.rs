use alloc::{alloc::alloc, alloc::Layout, boxed::Box};
use core::{fmt::Debug, mem::size_of};

const THREAD_STACK_SIZE: usize = 4096 * 1024;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ThreadRegisters {
    rbp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

#[derive(Debug)]
pub struct Thread<'a> {
    stack: Box<[u8; THREAD_STACK_SIZE]>,
    pub registers: &'a ThreadRegisters,
}

impl<'a> Thread<'a> {
    pub fn new(entry_point: fn() -> !) -> Self {
        let stack = unsafe {
            let ptr = alloc(Layout::new::<[u8; THREAD_STACK_SIZE]>()) as *mut [u8; THREAD_STACK_SIZE];
            Box::from_raw(ptr)
        };
        let ptr = ((stack.as_ptr() as usize & !0x15) + THREAD_STACK_SIZE - size_of::<ThreadRegisters>()) as *mut ThreadRegisters;
        let mut registers = unsafe { &mut *ptr };
        registers.rflags = 0x202;
        registers.cs = 0x8;
        registers.rip = entry_point as u64;
        registers.rsp = ptr as u64;
        Self { stack, registers }
    }
    pub fn stack_top(&self) -> usize {
        (self.stack.as_ptr() as usize & !0x15) + THREAD_STACK_SIZE - size_of::<ThreadRegisters>()
    }
}
