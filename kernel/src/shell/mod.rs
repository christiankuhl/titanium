use crate::asm::idle;
use crate::print;

pub mod vga_buffer;

pub fn start() -> ! {
    print!("> ");
    idle();
}

struct Shell {
    
}