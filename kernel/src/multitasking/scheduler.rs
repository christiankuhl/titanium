use core::ptr::NonNull;

use super::thread::{Thread, ThreadRegisters};

#[derive(Debug)]
pub struct Scheduler<'a> {
    num_threads: usize,
    current_thread: Option<usize>,
    threads: [Option<Thread<'a>>; 3],
    started: bool,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        const INIT: Option<Thread> = None;
        Self { num_threads: 0, current_thread: None, threads: [None, None, None], started: false }
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

    pub fn switch_thread(&mut self, cpu_state: &'a ThreadRegisters) -> &'a ThreadRegisters {
        if self.num_threads == 0 {
            return cpu_state;
        }
        if let Some(mut thread) = self.current_thread {
            if self.started {
                self.threads[thread].as_mut().unwrap().registers = cpu_state;
            } else {
                self.started = true;
            }
            thread = (thread + 1) % self.num_threads;
            self.current_thread = Some(thread);
            return self.threads[thread].as_ref().unwrap().registers;
        } else {
            panic!("No active tasks!")
        }
    }
}
