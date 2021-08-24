use lazy_static::lazy_static;

pub mod thread;
mod scheduler;

pub use thread::ThreadRegisters;
use scheduler::Scheduler;

lazy_static! {
    pub static ref SCHEDULER: spin::Mutex<Scheduler> = spin::Mutex::new(Scheduler::new());
}
