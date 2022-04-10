use crate::asm::outb;

const TIMER_BASE_FREQ: u64 = 1_193_182;
const INTERRUPT_FREQ: u64 = 1_000;

pub fn init() {
    let divisor = TIMER_BASE_FREQ / INTERRUPT_FREQ;
    unsafe {
        outb(0x43, 0x52);
        outb(0x40, (divisor & 0xff) as u8);
        outb(0x40, ((divisor >> 8) & 0xff) as u8);
    }
}
