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

fn banner(title: &str) {
    use AnsiColors::*;
    debugprintln!("{}", Yellow.text("\n╔══════════════════════════════════════════════════════════════════════════════╗"));
    debugprintln!("{}", Yellow.text(&alloc::format!("║ {:<76} ║", title)));
    debugprintln!("{}", Yellow.text("╚══════════════════════════════════════════════════════════════════════════════╝"));
}

pub fn test_runner_with_title(tests: &[&dyn Testable], title: &str) {
    banner(title);
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

pub fn single_test(tests: &[&dyn Testable]) {
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
    #[cfg(not(feature = "qemu_test_should_panic"))]
    fn run(&self) {
        use AnsiColors::*;
        debugprint!("{}...\t", core::any::type_name::<T>());
        self();
        debugprintln!("{}", Green.text("[ok]"));
    }
    #[cfg(feature = "qemu_test_should_panic")]
    fn run(&self) {
        use AnsiColors::*;
        debugprint!("{}...\t", core::any::type_name::<T>());
        self();
        debugprintln!("{}", Red.text("[failed]"));
    }
}

pub fn panic_handler(info: &PanicInfo) -> ! {
    use AnsiColors::*;
    debugprintln!("{}", Red.text("[failed]\n"));
    debugprintln!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    idle()
}

pub fn should_panic(_info: &PanicInfo) -> ! {
    use AnsiColors::*;
    debugprint!("{}", Green.text("[ok]\n"));
    exit_qemu(QemuExitCode::Success);
    idle()
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