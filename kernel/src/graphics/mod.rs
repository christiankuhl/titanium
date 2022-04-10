use crate::drivers::vga::{ScreenBuffer, SCREEN_WIDTH, VGA};
use alloc::boxed::Box;

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
pub struct Colour(u8, u8, u8, u8);

impl Colour {
    pub const fn black() -> Self {
        Colour(0, 0, 0, 0)
    }
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(b, g, r, a)
    }
    pub const fn red() -> Self {
        Self(0, 0, 0xff, 0xff)
    }
    pub const fn green() -> Self {
        Self(0, 0xff, 0, 0xff)
    }
    pub const fn blue() -> Self {
        Self(0xff, 0, 0, 0xff)
    }
    pub const fn titanium_bg() -> Self {
        Self(0x1e, 0x05, 0x00, 0xff)
    }
    pub const fn titanium_fg() -> Self {
        Self(0xba, 0x8f, 0x00, 0xff)
    }
}

pub struct Painter {
    back_buffer: Box<ScreenBuffer>,
}

impl Painter {
    pub fn new() -> Self {
        Self { back_buffer: ScreenBuffer::new() }
    }
    pub fn paint(&self) {
        let mut vga = VGA.lock();
        vga.flip(&*self.back_buffer);
    }
    pub fn fill_background(&mut self, colour: Colour) {
        self.back_buffer.fill(colour);
    }
    pub fn fill_rectangle(&mut self, rect: Rectangle, colour: Colour) {
        for dy in 0..rect.height() + 1 {
            for dx in 0..rect.width() + 1 {
                self.back_buffer[(rect.top() + dy) * SCREEN_WIDTH as usize + rect.left() + dx] = colour;
            }
        }
    }
    pub fn clear(&mut self) {
        self.back_buffer.clear();
    }
}

pub struct Rectangle {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

impl Rectangle {
    pub fn top(&self) -> usize {
        self.y0
    }
    pub fn left(&self) -> usize {
        self.x0
    }
    pub fn height(&self) -> usize {
        self.y1 - self.y0
    }
    pub fn width(&self) -> usize {
        self.x1 - self.x0
    }
}
