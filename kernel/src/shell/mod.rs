use crate::asm::idle;
use crate::{print, println};

pub mod vga_buffer;

pub fn start() -> ! {
    println!("Hello, world!\nHow are you on this most glorious of days?\n");
    print!("> ");
    idle();
}

struct Shell {}
