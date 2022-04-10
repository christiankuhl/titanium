mod clock;
pub mod vga_text_buffer;

use crate::{print, println};
use clock::Clock;

pub fn start() -> ! {
    let mut shell = Shell::new();
    shell.repl()
}

struct Shell {
    clock: Clock,
}

impl Shell {
    fn new() -> Self {
        Self { clock: Clock::new() }
    }
    fn repl(&mut self) -> ! {
        println!("Hello, world!\nHow are you on this most glorious of days?\n");
        print!("> ");
        self.clock.draw();
        loop {
            self.clock.update();
        }
    }
}
