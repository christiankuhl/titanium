use pic8259::ChainedPics;
use spin;

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn enable_interrupt(irq: u8) {
    let mut pics = PICS.lock();
    unsafe {
        let mut masks = pics.read_masks();
        if irq < 8 {
            masks[0] &= !(1 << irq)
        } else {
            masks[1] &= !(1 << (irq - 8))
        }
        pics.write_masks(masks[0], masks[1]);
    }
}
