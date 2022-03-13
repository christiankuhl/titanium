#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use kernel::*;

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info: &MultibootInfo) -> ! {
    log!("Entering Rust kernel...");

    init(multiboot_info);

    #[cfg(test)]
    test_main();

    let idle_thread = multitasking::thread::Thread::new(0, idle);
    let shell = multitasking::thread::Thread::new(1, shell::start);
    {
        let mut scheduler = multitasking::SCHEDULER.lock();
        scheduler.add_thread(idle_thread);
        scheduler.add_thread(shell);
        scheduler.start();
    }

    idle();
}

fn test_runner(tests: &[&dyn testing::Testable]) {
    testing::test_runner_with_title(tests, "Testing basic boot...")
}

#[test_case]
fn basic_boot() {
    assert!(true);
}
