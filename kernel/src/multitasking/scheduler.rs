use alloc::collections::VecDeque;

use super::thread::{Thread, ThreadRegisters};

#[derive(Debug)]
pub struct Scheduler {
    threads: VecDeque<Thread>,
    current_tid: Option<u64>,
    started: bool,
}

impl<'a> Scheduler {
    pub fn new() -> Self {
        Self { threads: VecDeque::new(), current_tid: None, started: false }
    }

    pub fn start(&mut self) {
        self.current_tid = Some(0);
    }

    pub fn add_thread(&mut self, thread: Thread) {
        self.threads.push_back(thread);
    }

    pub fn switch_thread(&mut self, cpu_state: *mut ThreadRegisters) -> *mut ThreadRegisters {
        if self.threads.len() == 0 {
            return cpu_state;
        }
        if self.started {
            let regs = self.threads.front_mut().unwrap().registers();
            unsafe { *regs = *cpu_state; }
        } else {
            self.started = true;
        }
        self.threads.rotate_left(1);
        let new_thread = self.threads.front_mut().unwrap();
        self.current_tid = Some(new_thread.tid);
        new_thread.registers()
    }
}
