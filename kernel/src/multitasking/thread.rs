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
    pub rflags: u64,
    rsp: u64,
    ss: u64,
}

#[derive(Debug)]
pub struct Thread {
    pub tid: u64,
    stack: Box<[u8; THREAD_STACK_SIZE]>,
    ptr: u64,
}

impl Thread {
    pub fn new(tid: u64, entry_point: fn() -> !) -> Self {
        let stack = unsafe {
            let ptr = alloc(Layout::new::<[u8; THREAD_STACK_SIZE]>()) as *mut [u8; THREAD_STACK_SIZE];
            Box::from_raw(ptr)
        };
        let registers = ((stack.as_ptr() as usize & !0x15) + THREAD_STACK_SIZE - size_of::<ThreadRegisters>()) as *mut ThreadRegisters;
        unsafe {
            (*registers).rflags = 0x202;
            (*registers).cs = 0x8;
            (*registers).rip = entry_point as u64;
            (*registers).rsp = registers as u64;
        }
        Self { tid, stack, ptr: registers as u64 }
    }
    pub fn registers(&mut self) -> *mut ThreadRegisters {
        return self.ptr as *mut ThreadRegisters
    }
}
