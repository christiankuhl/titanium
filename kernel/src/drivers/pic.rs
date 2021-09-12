use spin;

use crate::asm::{inb, outb};

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

const CMD_INIT: u8 = 0x11;
const CMD_END_OF_INTERRUPT: u8 = 0x20;
const MODE_8086: u8 = 0x01;

struct Pic {
    offset: u8,
    command: u16,
    data: u16,
}

impl Pic {
    fn handles_interrupt(&self, interupt_id: u8) -> bool {
        self.offset <= interupt_id && interupt_id < self.offset + 8
    }
    unsafe fn end_of_interrupt(&mut self) {
        self.write_cmd(CMD_END_OF_INTERRUPT);
    }
    unsafe fn read(&mut self) -> u8 {
        inb(self.data)
    }
    unsafe fn write_data(&mut self, value: u8) {
        outb(self.data, value)
    }
    unsafe fn write_cmd(&mut self, value: u8) {
        outb(self.command, value)
    }
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics: [Pic { offset: offset1, command: 0x20, data: 0x21 }, Pic { offset: offset2, command: 0xA0, data: 0xA1 }],
        }
    }

    pub unsafe fn initialize(&mut self) {
        let wait = || outb(0x80, 0);
        let saved_masks = self.read_masks();
        self.pics[0].write_cmd(CMD_INIT);
        wait();
        self.pics[1].write_cmd(CMD_INIT);
        wait();

        // Byte 1: Set up our base offsets.
        self.pics[0].write_data(self.pics[0].offset);
        wait();
        self.pics[1].write_data(self.pics[1].offset);
        wait();

        // Byte 2: Configure chaining between PIC1 and PIC2.
        self.pics[0].write_data(4);
        wait();
        self.pics[1].write_data(2);
        wait();

        self.pics[0].write_data(MODE_8086);
        wait();
        self.pics[1].write_data(MODE_8086);
        wait();

        self.write_masks(saved_masks[0], saved_masks[1]);
    }

    pub unsafe fn read_masks(&mut self) -> [u8; 2] {
        [self.pics[0].read(), self.pics[1].read()]
    }

    pub unsafe fn write_masks(&mut self, mask1: u8, mask2: u8) {
        self.pics[0].write_data(mask1);
        self.pics[1].write_data(mask2);
    }

    pub unsafe fn disable(&mut self) {
        self.write_masks(u8::MAX, u8::MAX)
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
    }

    /// Figure out which (if any) PICs in our chain need to know about this
    /// interrupt.  This is tricky, because all interrupts from `pics[1]`
    /// get chained through `pics[0]`.
    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.pics[1].handles_interrupt(interrupt_id) {
                self.pics[1].end_of_interrupt();
            }
            self.pics[0].end_of_interrupt();
        }
    }
}
