use core::panic::PanicInfo;

use crate::asm::{outl, idle};
use crate::{debugprintln, debugprint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        outl(0xf4, exit_code as u32);
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    debugprintln!("\nRunning {} tests...", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn(),
{
    fn run(&self) {
        debugprint!("{}...\t", core::any::type_name::<T>());
        self();
        debugprintln!("[ok]");
    }
}

pub fn panic_handler(info: &PanicInfo) -> ! {
    debugprintln!("[failed]\n");
    debugprintln!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    idle()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_handler(info)
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &crate::multiboot::MultibootInfo) -> ! {
    debugprintln!("--------------------------------------------------------------------------------");
    debugprintln!("| Unit tests for kernel library...                                             |");
    debugprintln!("--------------------------------------------------------------------------------");
    crate::init(multiboot_info);
    crate::run_tests();
    crate::idle();
}