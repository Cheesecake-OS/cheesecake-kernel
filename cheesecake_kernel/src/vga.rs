use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use core::fmt;
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
use spin::{Mutex, Once};

// Font config — change FONT_SIZE to Size14/Size18/Size22/Size32 as needed.
const FONT_WEIGHT: FontWeight = FontWeight::Regular;
const FONT_SIZE: RasterHeight = RasterHeight::Size16;

// Derived at runtime from the framebuffer info, but we need a fallback for
// the char width which is fixed per font config.
const CHAR_WIDTH: usize = get_raster_width(FONT_WEIGHT, FONT_SIZE);

/// 16-color palette (matches the original VGA Color enum values exactly).
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

impl Color {
    /// Returns the RGB triple for the classic CGA/VGA palette.
    const fn rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Black => (0, 0, 0),
            Color::Blue => (0, 0, 170),
            Color::Green => (0, 170, 0),
            Color::Cyan => (0, 170, 170),
            Color::Red => (170, 0, 0),
            Color::Magenta => (170, 0, 170),
            Color::Brown => (170, 85, 0),
            Color::LightGray => (170, 170, 170),
            Color::DarkGray => (85, 85, 85),
            Color::LightBlue => (85, 85, 255),
            Color::LightGreen => (85, 255, 85),
            Color::LightCyan => (85, 255, 255),
            Color::LightRed => (255, 85, 85),
            Color::Pink => (255, 85, 255),
            Color::Yellow => (255, 255, 85),
            Color::White => (255, 255, 255),
        }
    }
}

// --------------------------------------------------------------------------
// Writer
// --------------------------------------------------------------------------

pub struct Writer {
    fb: &'static mut [u8],
    info: FrameBufferInfo,
    x: usize, // pixel x
    y: usize, // pixel y (top of current text row)
    fg: Color,
    bg: Color,
}

impl Writer {
    fn new(fb: &'static mut FrameBuffer) -> Self {
        let info = fb.info();
        let buf = fb.buffer_mut();
        let mut w = Writer {
            fb: buf,
            info,
            x: 0,
            y: 0,
            fg: Color::LightGreen,
            bg: Color::Black,
        };
        w.clear();
        w
    }

    // ------------------------------------------------------------------
    // Low-level pixel write
    // ------------------------------------------------------------------

    /// Write a single pixel at (px, py) with given (r, g, b).
    #[inline(always)]
    fn set_pixel(&mut self, px: usize, py: usize, r: u8, g: u8, b: u8) {
        let stride = self.info.stride;
        let bpp = self.info.bytes_per_pixel;
        let offset = (py * stride + px) * bpp;
        if offset + bpp > self.fb.len() {
            return;
        }
        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.fb[offset] = r;
                self.fb[offset + 1] = g;
                self.fb[offset + 2] = b;
            }
            PixelFormat::Bgr => {
                self.fb[offset] = b;
                self.fb[offset + 1] = g;
                self.fb[offset + 2] = r;
            }
            PixelFormat::U8 => {
                // Luminance: simple average
                self.fb[offset] = ((r as u16 + g as u16 + b as u16) / 3) as u8;
            }
            _ => {
                // Unknown format — write bytes as-is and hope for the best.
                self.fb[offset] = r;
                self.fb[offset + 1] = g;
                self.fb[offset + 2] = b;
            }
        }
    }

    // ------------------------------------------------------------------
    // Character rendering
    // ------------------------------------------------------------------

    fn char_height(&self) -> usize {
        FONT_SIZE as usize
    }

    fn char_width(&self) -> usize {
        CHAR_WIDTH
    }

    fn render_char(&mut self, c: char) {
        // Fall back to '?' for missing glyphs.
        let raster = get_raster(c, FONT_WEIGHT, FONT_SIZE)
            .or_else(|| get_raster('?', FONT_WEIGHT, FONT_SIZE));

        let (fg_r, fg_g, fg_b) = self.fg.rgb();
        let (bg_r, bg_g, bg_b) = self.bg.rgb();

        let cx = self.x;
        let cy = self.y;

        if let Some(raster) = raster {
            for (row_idx, row) in raster.raster().iter().enumerate() {
                for (col_idx, &intensity) in row.iter().enumerate() {
                    // Alpha-blend: intensity 0 = full bg, 255 = full fg.
                    let a = intensity as u16;
                    let r = ((fg_r as u16 * a + bg_r as u16 * (255 - a)) / 255) as u8;
                    let g = ((fg_g as u16 * a + bg_g as u16 * (255 - a)) / 255) as u8;
                    let b = ((fg_b as u16 * a + bg_b as u16 * (255 - a)) / 255) as u8;
                    self.set_pixel(cx + col_idx, cy + row_idx, r, g, b);
                }
            }
        } else {
            // Fill glyph cell with bg color when raster unavailable.
            let ch = self.char_height();
            let cw = self.char_width();
            for row_idx in 0..ch {
                for col_idx in 0..cw {
                    self.set_pixel(cx + col_idx, cy + row_idx, bg_r, bg_g, bg_b);
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Scrolling
    // ------------------------------------------------------------------

    fn scroll_up(&mut self) {
        let ch = self.char_height();
        let stride = self.info.stride;
        let bpp = self.info.bytes_per_pixel;
        let width = self.info.width;
        let height = self.info.height;

        // Shift every row of pixels up by one text row.
        let row_bytes = stride * bpp;
        let shift = ch * row_bytes;
        let total = height * row_bytes;

        self.fb.copy_within(shift..total, 0);

        // Clear the newly exposed bottom text row.
        let (bg_r, bg_g, bg_b) = self.bg.rgb();
        let clear_start = (height - ch) * row_bytes;
        for py in (height - ch)..height {
            for px in 0..width {
                let off = (py * stride + px) * bpp;
                if off + bpp > self.fb.len() {
                    break;
                }
                match self.info.pixel_format {
                    PixelFormat::Rgb => {
                        self.fb[off] = bg_r;
                        self.fb[off + 1] = bg_g;
                        self.fb[off + 2] = bg_b;
                    }
                    PixelFormat::Bgr => {
                        self.fb[off] = bg_b;
                        self.fb[off + 1] = bg_g;
                        self.fb[off + 2] = bg_r;
                    }
                    PixelFormat::U8 => {
                        self.fb[off] = ((bg_r as u16 + bg_g as u16 + bg_b as u16) / 3) as u8;
                    }
                    _ => {
                        self.fb[off] = bg_r;
                        self.fb[off + 1] = bg_g;
                        self.fb[off + 2] = bg_b;
                    }
                }
            }
        }
        let _ = clear_start; // suppress unused warning
    }

    // ------------------------------------------------------------------
    // Public write primitives (called by macros/public API)
    // ------------------------------------------------------------------

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => {
                self.x = 0;
            }
            byte => {
                if self.x + self.char_width() > self.info.width {
                    self.newline();
                }
                self.render_char(byte as char);
                self.x += self.char_width();
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => self.newline(),
                '\r' => {
                    self.x = 0;
                }
                c => {
                    if self.x + self.char_width() > self.info.width {
                        self.newline();
                    }
                    self.render_char(c);
                    self.x += self.char_width();
                }
            }
        }
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => {
                self.x = 0;
            }
            c => {
                if self.x + self.char_width() > self.info.width {
                    self.newline();
                }
                self.render_char(c);
                self.x += self.char_width();
            }
        }
    }

    fn newline(&mut self) {
        self.x = 0;
        self.y += self.char_height();
        if self.y + self.char_height() > self.info.height {
            self.scroll_up();
            self.y = self.info.height - self.char_height();
        }
    }

    // ------------------------------------------------------------------
    // Public API surface (matches the original vga.rs)
    // ------------------------------------------------------------------

    pub fn clear(&mut self) {
        let (bg_r, bg_g, bg_b) = self.bg.rgb();
        let stride = self.info.stride;
        let bpp = self.info.bytes_per_pixel;
        let width = self.info.width;
        let height = self.info.height;

        for py in 0..height {
            for px in 0..width {
                let off = (py * stride + px) * bpp;
                if off + bpp > self.fb.len() {
                    break;
                }
                match self.info.pixel_format {
                    PixelFormat::Rgb => {
                        self.fb[off] = bg_r;
                        self.fb[off + 1] = bg_g;
                        self.fb[off + 2] = bg_b;
                    }
                    PixelFormat::Bgr => {
                        self.fb[off] = bg_b;
                        self.fb[off + 1] = bg_g;
                        self.fb[off + 2] = bg_r;
                    }
                    PixelFormat::U8 => {
                        self.fb[off] = ((bg_r as u16 + bg_g as u16 + bg_b as u16) / 3) as u8;
                    }
                    _ => {
                        self.fb[off] = bg_r;
                        self.fb[off + 1] = bg_g;
                        self.fb[off + 2] = bg_b;
                    }
                }
            }
        }
        self.x = 0;
        self.y = 0;
    }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.fg = fg;
        self.bg = bg;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Writer::write_str(self, s);
        Ok(())
    }
}

// --------------------------------------------------------------------------
// Global singleton — must be initialised once from kernel_main via init().
// --------------------------------------------------------------------------

static WRITER: Once<Mutex<Writer>> = Once::new();

/// Call this exactly once from `kernel_main`, passing the `FrameBuffer`
/// out of `BootInfo`.
///
/// ```rust
/// // in kernel_main:
/// vga::init(boot_info.framebuffer.as_mut().unwrap());
/// ```
pub fn init(fb: &'static mut FrameBuffer) {
    WRITER.call_once(|| Mutex::new(Writer::new(fb)));
}

#[inline]
fn writer() -> &'static Mutex<Writer> {
    WRITER.get().expect("vga::init() not called")
}

// --------------------------------------------------------------------------
// Public free functions — identical API to the original module
// --------------------------------------------------------------------------

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    writer().lock().write_fmt(args).unwrap();
}

pub fn print(s: &str) {
    writer().lock().write_str(s);
}

pub fn print_char(c: char) {
    writer().lock().write_char(c);
}

pub fn clear() {
    writer().lock().clear();
}

pub fn set_color(fg: Color, bg: Color) {
    writer().lock().set_color(fg, bg);
}

// --------------------------------------------------------------------------
// Macros — identical to the originals
// --------------------------------------------------------------------------

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn print_ok(msg: &str) {
    let mut w = writer().lock();
    let saved_fg = w.fg;
    w.fg = Color::LightGreen;
    w.write_str("[ OK ] ");
    w.fg = saved_fg;
    w.write_str(msg);
    w.write_str("\n");
}

pub fn print_err(msg: &str) {
    let mut w = writer().lock();
    let saved_fg = w.fg;
    w.fg = Color::LightRed;
    w.write_str("[ ERR ] ");
    w.fg = saved_fg;
    w.write_str(msg);
    w.write_str("\n");
}

pub fn print_info(msg: &str) {
    let mut w = writer().lock();
    let saved_fg = w.fg;
    w.fg = Color::LightCyan;
    w.write_str("[ INFO ] ");
    w.fg = saved_fg;
    w.write_str(msg);
    w.write_str("\n");
}
