use super::port::Port;
use crate::asm::without_interrupts;
use lazy_static::lazy_static;
use spin::Mutex;

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop()
        }
    };
}

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub struct SerialPort {
    data: Port<u8>,
    int_en: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_sts: Port<u8>,
}

impl SerialPort {
    /// Creates a new serial port interface on the given I/O port.
    /// This function is unsafe because the caller must ensure that the given base address
    /// really points to a serial port device.
    pub unsafe fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            int_en: Port::new(base + 1),
            fifo_ctrl: Port::new(base + 2),
            line_ctrl: Port::new(base + 3),
            modem_ctrl: Port::new(base + 4),
            line_sts: Port::new(base + 5),
        }
    }
    /// Initializes the serial port.
    /// The default configuration of [38400/8-N-1](https://en.wikipedia.org/wiki/8-N-1) is used.
    pub fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.int_en.write(0x00);
            // Enable DLAB
            self.line_ctrl.write(0x80);
            // Set maximum speed to 38400 bps by configuring DLL and DLM
            self.data.write(0x03);
            self.int_en.write(0x00);
            // Disable DLAB and set data word length to 8 bits
            self.line_ctrl.write(0x03);
            // Enable FIFO, clear TX/RX queues and
            // set interrupt watermark at 14 bytes
            self.fifo_ctrl.write(0xC7);
            // Mark data terminal ready, signal request to send
            // and enable auxilliary output #2 (used as interrupt line for CPU)
            self.modem_ctrl.write(0x0B);
            // Enable interrupts
            self.int_en.write(0x01);
        }
    }

    pub fn send(&mut self, data: u8) {
        unsafe {
            match data {
                8 | 0x7F => {
                    wait_for!(self.output_empty());
                    self.data.write(8);
                    wait_for!(self.output_empty());
                    self.data.write(b' ');
                    wait_for!(self.output_empty());
                    self.data.write(8)
                }
                _ => {
                    wait_for!(self.output_empty());
                    self.data.write(data);
                }
            }
        }
    }

    fn output_empty(&self) -> bool {
        unsafe { self.line_sts.read() & (1 << 5) > 0 }
    }
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! debugprint {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*));
    };
}

/// Prints unconditionally to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! debugprintln {
    () => ($crate::debugprint!("\n"));
    ($fmt:expr) => ($crate::debugprint!(concat!($fmt, "\r\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::debugprint!(
        concat!($fmt, "\r\n"), $($arg)*));
}

/// Prints to the host through the serial interface, appending a newline, except when in test
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => (
        #[cfg(not(any(feature = "qemu_test", feature = "qemu_test_should_panic")))]
        $crate::debugprintln!($($arg)*)
    );
}
