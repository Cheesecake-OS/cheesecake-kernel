use uart_16550::SerialPort;
use spin::{Mutex, Once};
use core::fmt;

static SERIAL1_ONCE: Once<Mutex<SerialPort>> = Once::new();

fn serial() -> &'static Mutex<SerialPort> {
    SERIAL1_ONCE.call_once(|| {
        let mut port = unsafe { SerialPort::new(0x3F8) };
        port.init();
        Mutex::new(port)
    })
}

pub fn init() {
    serial();
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    serial().lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
