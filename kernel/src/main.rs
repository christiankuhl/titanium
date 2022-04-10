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

    // test_animation();

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

fn f(t: usize) -> usize {
    t.pow(2) * (100 - t).pow(2) / 12500
}

fn test_animation() {
    use crate::graphics::{Colour, Painter, Rectangle};
    let mut p = Painter::new();
    for t in 0..100 {
        let r = Rectangle { x0: 100 + f(t), y0: 100 + f(t), x1: 200 + f(t), y1: 200 + f(t) };
        p.fill_background(Colour::titanium_bg());
        p.fill_rectangle(r, Colour::titanium_fg());
        p.paint();
        for _ in 0..500 {
            log!("Foo");
        }
    }
}

#[test_case]
fn basic_boot() {
    assert!(true);
}

// const q: Colour = Colour::rgba(0x03, 0x0f, 0x2e, 0xff);
