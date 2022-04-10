use crate::time::Timestamp;
use alloc::format;
use super::vga_text_buffer;

pub struct Clock {
    time: Timestamp
}

impl Clock {
    pub fn new() -> Self {
        Self { time: Timestamp::now() }
    }
    pub fn update(&mut self) {
        let new_time = Timestamp::now();
        if new_time == self.time { return };
        self.time = new_time;
        self.draw();
    }
    pub fn draw(&self) {
        let t = format!("{}", self.time);
        vga_text_buffer::print_at(&format!("{:>80}", t), 0, 0);
    }
}