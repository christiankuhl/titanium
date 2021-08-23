use alloc::boxed::Box;
use core::{fmt::Debug, mem::size_of};
use lazy_static::lazy_static;

use crate::{interrupts::InterruptStackFrame, println};

const THREAD_STACK_SIZE: usize = 4096;

lazy_static! {
    pub static ref SCHEDULER: spin::Mutex<Scheduler<'static>> = spin::Mutex::new(Scheduler::new());
}

#[derive(Debug)]
pub struct Thread<'a> {
    stack: Box<[u8; 4096]>,
    cpu_state: &'a CPUState,
}

impl<'a> Thread<'a> {
    pub fn new(entry_point: fn()) -> Self {
        let stack = Box::new([0; THREAD_STACK_SIZE]);
        let ptr = ((stack.as_ptr() as usize & !0x15) + THREAD_STACK_SIZE - size_of::<CPUState>()) as *mut CPUState;
        let mut cpu_state = unsafe { &mut *ptr };
        cpu_state.rflags = 0x202;
        cpu_state.cs = 0x8;
        cpu_state.rip = entry_point as u64;
        cpu_state.rsp = ptr as u64;
        Self { stack, cpu_state }
    }
}

#[derive(Debug)]
pub struct Scheduler<'a> {
    num_threads: usize,
    current_thread: Option<usize>,
    threads: [Option<Thread<'a>>; 2],
    started: bool,
}

impl<'a> Scheduler<'a> {
    fn new() -> Self {
        // const INIT: Option<Thread> = None;
        Self { num_threads: 0, current_thread: None, threads: [None, None], started: false }
    }

    pub fn add_thread(&mut self, thread: Thread<'a>) {
        if self.num_threads >= 255 {
            panic!("No more tasks available!")
        } else {
            self.threads[self.num_threads] = Some(thread);
            self.num_threads += 1;
        }
    }

    pub fn start(&mut self) {
        self.current_thread = Some(0);
    }

    pub fn switch_thread(&mut self, cpu_state: &'a CPUState) -> &'a CPUState {
        if self.num_threads == 0 {
            return cpu_state;
        }
        if let Some(mut thread) = self.current_thread {
            if self.started {
                self.threads[thread].as_mut().unwrap().cpu_state = cpu_state;
            } else {
                self.started = true;
            }
            thread = (thread + 1) % self.num_threads;
            self.current_thread = Some(thread);
            return self.threads[thread].as_ref().unwrap().cpu_state;
        } else {
            panic!("No active tasks!")
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct CPUState {
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
