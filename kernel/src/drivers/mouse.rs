use lazy_static::lazy_static;

use crate::shell::vga_buffer::WRITER;

use super::port::Port;

lazy_static! {
    pub static ref MOUSE: spin::Mutex<Mouse> = spin::Mutex::new(Mouse::new());
}

static mut MOUSE_UNINIT: bool = true;
static mut MOUSE_X: i8 = -1;
static mut MOUSE_Y: i8 = -1;

pub struct Mouse {
    buffer: [i8; 3],
    offset: usize,
    buttons: u8,
}
enum EventType {
    Data,
    Signal,
}

pub fn init_mouse() {
    let mut initializer = MouseInit::new();
    unsafe {
        initializer.init();
    }
}

use EventType::*;

impl Mouse {
    pub fn new() -> Self {
        Self { buffer: [0, 0, 0], offset: 0, buttons: 0 }
    }
    pub fn add_byte(&mut self, data: i8) {
        self.buffer[self.offset] = data;
        self.offset = (self.offset + 1) % 3;
    }
    pub fn event(&self) -> Option<MouseEvent> {
        use MouseEvent::*;
        if self.offset % 3 != 0 {
            return None;
        }
        Some(Move(self.buffer[1], self.buffer[2]))
    }
}

pub enum MouseEvent {
    Move(i8, i8),
    LeftClick,
    RightClick,
}

struct MouseInit {
    ctrl_port: Port<u8>,
    data_port: Port<u8>,
}

impl MouseInit {
    pub fn new() -> Self {
        Self { ctrl_port: Port::new(0x64), data_port: Port::new(0x60) }
    }
    unsafe fn wait_for(&mut self, event: &EventType) {
        let mut timeout = 100000;
        while timeout > 0 {
            match event {
                EventType::Data => {
                    if (self.ctrl_port.read() & 1) == 1 {
                        return;
                    }
                }
                EventType::Signal => {
                    if (self.ctrl_port.read() & 2) == 0 {
                        return;
                    }
                }
            }
            timeout -= 1;
        }
    }

    pub unsafe fn init(&mut self) {
        //Enable the auxiliary mouse device
        self.wait_for(&Signal);
        self.ctrl_port.write(0xa8u8);
        //Enable the interrupts
        self.wait_for(&Signal);
        self.ctrl_port.write(0x20u8);
        self.wait_for(&Data);
        let status: u8 = self.data_port.read() | 2;
        self.wait_for(&Signal);
        self.ctrl_port.write(0x60u8);
        self.wait_for(&Signal);
        self.data_port.write(status);
        // Tell the mouse to use default settings
        self.write(0xf6);
        self.read(); // Acknowledge
                     // Enable the mouse
        self.write(0xf4);
        self.read(); // Acknowledge
    }
    unsafe fn write(&mut self, command: u8) {
        self.wait_for(&Signal);
        self.ctrl_port.write(0xd4u8);
        self.wait_for(&Signal);
        self.data_port.write(command);
    }
    fn read(&mut self) -> u8 {
        unsafe {
            self.wait_for(&Data);
            self.data_port.read()
        }
    }
}

pub unsafe fn move_mouse_cursor(dx: i8, dy: i8) {
    if dx == 0 && dy == 0 {
        return;
    }
    let mut screen = WRITER.lock();
    screen.invert(MOUSE_Y as usize, MOUSE_X as usize);
    MOUSE_X += dx / 2;
    if MOUSE_X < 0 {
        MOUSE_X = 0
    };
    if MOUSE_X >= 80 {
        MOUSE_X = 79
    };
    MOUSE_Y -= dy / 2;
    if MOUSE_Y < 0 {
        MOUSE_Y = 0
    };
    if MOUSE_Y >= 25 {
        MOUSE_Y = 24
    };
    screen.invert(MOUSE_Y as usize, MOUSE_X as usize);
}

#[inline]
pub fn position() -> (usize, usize) {
    unsafe { (MOUSE_X as usize, MOUSE_Y as usize) }
}
