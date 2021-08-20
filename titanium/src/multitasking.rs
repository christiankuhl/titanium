use alloc::boxed::Box;
use core::fmt::Debug;
use lazy_static::lazy_static;

use crate::println;
use crate::interrupts::InterruptStackFrame;

lazy_static! {
    pub static ref TASKMANAGER: spin::Mutex<TaskManager> = spin::Mutex::new(TaskManager::new());
}

#[derive(Debug)]
pub struct Task {
    stack: Box<[u8; 4096]>,
    cpu_state: InterruptStackFrame,
}

impl Task {
    pub fn new(entry_point: fn()) -> Self {
        let cpu_state = InterruptStackFrame {
            instruction_pointer: entry_point as u64,
            code_segment: 0x8,
            stack_pointer: 0,
            stack_segment: 0,
            cpu_flags: 0x202,
        };
        let stack = Box::new([0; 4096]);
        let mut task = Self { stack, cpu_state };
        let addr = task.stack.as_ptr() as u64 + 4096;
        task.cpu_state.stack_pointer = addr;
        task
    }
}

#[derive(Debug)]
pub struct TaskManager {
    num_tasks: usize,
    current_task: Option<usize>,
    tasks: [Option<Task>; 256],
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

    // pub fn switch_task(&mut self, cpu_state: InterruptStackFrame) -> InterruptStackFrame {
    //     println!("Switch to {:?}", self.current_task);
    //     if self.num_tasks == 0 {
    //         return cpu_state;
    //     }
    //     if let Some(mut task) = self.current_task {
    //         self.tasks[task].as_mut().unwrap().cpu_state = cpu_state;
    //         task = (task + 1) % self.num_tasks;
    //         return self.tasks[task].as_ref().unwrap().cpu_state;
    //     } else {
    //         panic!("No active tasks!")
    //     }
    // }
}
