use core::fmt;
use spin::{Mutex, Once};
use volatile::Volatile;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER: usize = 0xb8000;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> Self {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii: u8,
    color: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; VGA_WIDTH]; VGA_HEIGHT],
}

pub struct Writer {
    col: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.col >= VGA_WIDTH {
                    self.newline();
                }
                let row = VGA_HEIGHT - 1;
                let col = self.col;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii: byte,
                    color: self.color,
                });
                self.col += 1;
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn newline(&mut self) {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let ch = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(ch);
            }
        }
        self.clear_row(VGA_HEIGHT - 1);
        self.col = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii: b' ',
            color: self.color,
        };
        for col in 0..VGA_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear(&mut self) {
        for row in 0..VGA_HEIGHT {
            self.clear_row(row);
        }
        self.col = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

static WRITER_ONCE: Once<Mutex<Writer>> = Once::new();

fn writer() -> &'static Mutex<Writer> {
    WRITER_ONCE.call_once(|| {
        Mutex::new(Writer {
            col: 0,
            color: ColorCode::new(Color::LightGreen, Color::Black),
            buffer: unsafe { &mut *(VGA_BUFFER as *mut Buffer) },
        })
    })
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    writer().lock().write_fmt(args).unwrap();
}

pub fn print(s: &str) {
    writer().lock().write_str(s);
}

pub fn clear() {
    writer().lock().clear();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}