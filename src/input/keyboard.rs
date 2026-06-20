use crate::serial_println;
use spin::Mutex;
use x86_64::instructions::port::Port;

// PS2 keyboard
const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;

static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new());

pub struct Keyboard {
    data: Port<u8>,
    shift: bool,
    caps: bool,
}

impl Keyboard {
    pub const fn new() -> Self {
        Keyboard {
            data: Port::new(DATA_PORT),
            shift: false,
            caps: false,
        }
    }

    pub fn read_scancode(&mut self) -> Option<u8> {
        let mut status: Port<u8> = Port::new(STATUS_PORT);
        let s = unsafe { status.read() };
        if s & 0x01 != 0 {
            Some(unsafe { self.data.read() })
        } else {
            None
        }
    }

    pub fn process(&mut self) -> Option<char> {
        let sc = self.read_scancode()?;

        match sc {
            0xAA | 0xB6 => {
                self.shift = false;
                None
            } // shift released
            0x2A | 0x36 => {
                self.shift = true;
                None
            } // shift pressed
            0x3A => {
                self.caps = !self.caps;
                None
            } // caps
            0x01 => {
                serial_println!("ESC");
                None
            }
            code if code < 0x80 => {
                let c = scancode_to_char(code, self.shift ^ self.caps);
                if let Some(ch) = c {
                    serial_println!("[KEY] '{}'", ch);
                }
                c
            }
            _ => None, // key release or unknown
        }
    }
}

pub fn poll() -> Option<char> {
    KEYBOARD.lock().process()
}

fn scancode_to_char(sc: u8, shifted: bool) -> Option<char> {
    let normal = [
        '\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08', '\t', 'q',
        'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '\0', 'a', 's', 'd', 'f', 'g',
        'h', 'j', 'k', 'l', ';', '\'', '`', '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',',
        '.', '/', '\0', '*', '\0', ' ',
    ];
    let shifted_map = [
        '\0', '\0', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\x08', '\t', 'Q',
        'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n', '\0', 'A', 'S', 'D', 'F', 'G',
        'H', 'J', 'K', 'L', ':', '"', '~', '\0', '|', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>',
        '?', '\0', '*', '\0', ' ',
    ];

    let map = if shifted { &shifted_map } else { &normal };
    let idx = sc as usize;
    if idx < map.len() {
        let c = map[idx];
        if c != '\0' {
            Some(c)
        } else {
            None
        }
    } else {
        None
    }
}
