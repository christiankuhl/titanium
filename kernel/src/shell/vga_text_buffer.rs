use crate::asm::without_interrupts;
use crate::drivers::mouse;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = {
        let mut writer = Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::LightBlue, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        };
        writer.clear();
        Mutex::new(writer)
    };
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
    fn invert(&mut self) {
        self.0 = (self.0 & 0xf0) >> 4 | (self.0 & 0xf) << 4;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    pub fn invert(&mut self) {
        self.color_code.invert();
    }
    pub fn new(byte: u8) -> Self {
        ScreenChar { ascii_character: byte, color_code: ColorCode::new(Color::LightBlue, Color::Black) }
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_screen_char(&mut self, char: ScreenChar, row: usize, col: usize) {
        if row >= BUFFER_HEIGHT || col >= BUFFER_WIDTH {
            return;
        }
        let mut character = char;
        let pos = mouse::position();
        if (col, row) == pos {
            character.invert();
        }
        self.buffer.chars[row][col].write(char);
    }
    pub fn invert(&mut self, row: usize, col: usize) {
        if row >= BUFFER_HEIGHT || col >= BUFFER_WIDTH {
            return;
        }
        self.buffer.chars[row][col].update(|char| char.invert());
    }
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => {
                self.column_position = 0;
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                let mut character = ScreenChar { ascii_character: byte, color_code };
                let pos = mouse::position();
                if (col, row) == pos {
                    character.invert();
                }
                self.buffer.chars[row][col].write(character);
                self.column_position += 1;
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' | b'\r' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn write_string_to(&mut self, s: &str, row: usize, col: usize) {
        let mut row = row;
        let mut col = col;
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' | b'\r' => self.write_screen_char(ScreenChar::new(byte), row, col),
                // not part of printable ASCII range
                _ => self.write_screen_char(ScreenChar::new(0xfe), row, col),
            }
            col += 1;
            if col >= BUFFER_WIDTH {
                row += 1;
                col = 0;
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let mut character = self.buffer.chars[row][col].read();
                let pos = mouse::position();
                if (col, row) == pos || (col, row - 1) == pos {
                    character.invert();
                }
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar { ascii_character: b' ', color_code: self.color_code };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::shell::vga_text_buffer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn print_at(s: &str, row: usize, column: usize) {
    without_interrupts(|| {
        WRITER.lock().write_string_to(s, row, column);
    });
}

#[test_case]
fn print_doesnt_panic() {
    println!("This worked!");
}

#[test_case]
fn println_output_eq_input() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
