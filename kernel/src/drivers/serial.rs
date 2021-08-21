use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use crate::asm::without_interrupts;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
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

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! debugprintln {
    () => ($crate::debugprint!("\n"));
    ($fmt:expr) => ($crate::debugprint!(concat!($fmt, "\r\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::debugprint!(
        concat!($fmt, "\r\n"), $($arg)*));
}
