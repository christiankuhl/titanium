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

pub fn test_runner_with_title(tests: &[&dyn Testable], title: &str) {
    use AnsiColors::*;
    debugprintln!("{}", Yellow.text("\n╔══════════════════════════════════════════════════════════════════════════════╗"));
    debugprintln!("{}", Yellow.text(&alloc::format!("║ {:<76} ║", title)));
    debugprintln!("{}", Yellow.text("╚══════════════════════════════════════════════════════════════════════════════╝"));
    debugprintln!("\nRunning {} tests...", tests.len());
    for test in tests {
        test.run();
    }
    debugprintln!();
    exit_qemu(QemuExitCode::Success);
}

pub fn test_runner(tests: &[&dyn Testable]) {
    test_runner_with_title(tests, "Unit tests for kernel library...")
}


pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn(),
{
    fn run(&self) {
        use AnsiColors::*;
        debugprint!("{}...\t", core::any::type_name::<T>());
        self();
        debugprintln!("{}", Green.text("[ok]"));
    }
}

pub fn panic_handler(info: &PanicInfo) -> ! {
    use AnsiColors::*;
    debugprintln!("{}", Red.text("[failed]\n"));
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
    crate::init(multiboot_info);
    crate::run_tests();
    crate::idle();
}


#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AnsiColors {
    Reset = 0,
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
}

impl AnsiColors {
    pub fn text(&self, text: &str) -> alloc::string::String {
        alloc::format!("\u{001b}[{}m{}\u{001b}[{}m", *self as u8, text, Self::Reset as u8)
    }
}