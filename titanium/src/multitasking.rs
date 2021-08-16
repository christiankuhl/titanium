use core::fmt::Debug;
use x86_64::{VirtAddr, structures::idt::InterruptStackFrame};
use alloc::boxed::Box;
use lazy_static::lazy_static;

use crate::println;

lazy_static! {
    pub static ref TASKMANAGER: spin::Mutex<TaskManager> = spin::Mutex::new(
        TaskManager::new()
    );
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct CPUState {
    pub instruction_pointer: VirtAddr,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: VirtAddr,
    pub stack_segment: u64,
}

impl CPUState {
    pub fn from_stack_frame(stack_fame: &InterruptStackFrame) -> Self {
        Self {
            instruction_pointer: stack_fame.instruction_pointer,
            code_segment: stack_fame.code_segment,
            cpu_flags: stack_fame.cpu_flags,
            stack_pointer: stack_fame.stack_pointer,
            stack_segment: stack_fame.stack_segment,
        }
    }
}

#[derive(Debug)]
pub struct Task {
    stack: Box<[u8; 4096]>,
    cpu_state: CPUState,
}

impl Task {
    pub fn new(entry_point: fn()) -> Self {
        let cpu_state = CPUState {
            instruction_pointer: VirtAddr::new(entry_point as u64),
            code_segment: 0x8,
            stack_pointer: VirtAddr::new(0),
            stack_segment: 0,
            cpu_flags: 0x202,
        };
        let stack = Box::new([0; 4096]);
        let mut task = Self { stack, cpu_state };
        let addr = task.stack.as_ptr() as u64 + 4096;
        task.cpu_state.stack_pointer = VirtAddr::new(addr);
        task
    }
}

#[derive(Debug)]
pub struct TaskManager {
    num_tasks: usize,
    current_task: Option<usize>,
    tasks: [Option<Task>; 256]
}

impl TaskManager {
    fn new() -> Self {
        const INIT: Option<Task> = None;
        Self { num_tasks: 0, current_task: None, tasks: [INIT; 256] }
    }

    pub fn add_task(&mut self, task: Task) {
        if self.num_tasks >= 255 {
            panic!("No more tasks available!")
        } else {
            self.tasks[self.num_tasks] = Some(task);
            self.num_tasks += 1;
        }
    }

    pub fn start(&mut self) {
        self.current_task = Some(0);
    }

    pub fn switch_task(&mut self, cpu_state: CPUState) -> CPUState {
        println!("Switch to {:?}", self.current_task);
        if self.num_tasks == 0 {
            return cpu_state
        }
        if let Some(mut task) = self.current_task {
            self.tasks[task].as_mut().unwrap().cpu_state = cpu_state;
            task = (task + 1) % self.num_tasks;
            return self.tasks[task].as_ref().unwrap().cpu_state;
        } else {
            panic!("No active tasks!")
        }
    }
}