use lazy_static::lazy_static;

mod scheduler;
pub mod thread;

use scheduler::Scheduler;
pub use thread::ThreadRegisters;

lazy_static! {
    pub static ref SCHEDULER: spin::Mutex<Scheduler> = spin::Mutex::new(Scheduler::new());
}
