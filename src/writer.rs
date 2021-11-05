use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use vga::colors::Color16;
use volatile::Volatile;
use x86_64::instructions::interrupts;

lazy_static! {
    pub static ref WRITER: Mutex<Writer<'static>> = Mutex::new(Writer {
        buf: unsafe {
            Volatile::new(core::slice::from_raw_parts_mut(
                vga::vga::VGA.lock().get_frame_buffer() as isize as *mut u8,
                80 * 25 * 2,
            ))
        },
        bg_color: Color16::Black,
        column_position: 0,
    });
}
pub struct Writer<'a> {
    pub bg_color: Color16,
    pub buf: Volatile<&'a mut [u8]>,
    pub column_position: usize,
}
impl Writer<'_> {
    pub fn write_byte(&mut self, byte: u8, color: Color16) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= 80 {
                    self.new_line();
                }
                let row = 24;
                let col = self.column_position;

                self.set_char(row, col, byte, color as u8);
                self.column_position += 1;
            }
        }
    }
    pub fn write_string(&mut self, str: &str, color: Color16) {
        for byte in str.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte, color),
                // not part of printable ASCII range
                _ => self.write_byte(b'\xDB', color),
            }
        }
    }
    pub fn set_char(&mut self, row: usize, col: usize, byte: u8, color: u8) {
        self.buf.index_mut(((row * 80) + col) * 2).write(byte);
        self.buf
            .index_mut((((row * 80) + col) * 2) + 1)
            .write(color as u8);
    }
    pub fn get_char(&mut self, row: usize, col: usize) -> (char, u8) {
        (
            self.buf.index(((row * 80) + col) * 2).read() as char,
            self.buf.index((((row * 80) + col) * 2) + 1).read(),
        )
    }
    fn clear_row(&mut self, row: usize) {
        for col in 0..80 {
            self.set_char(row, col, b' ', Color16::White as u8);
        }
    }

    fn new_line(&mut self) {
        for row in 1..25 {
            for col in 0..80 {
                let (character, color) = self.get_char(row, col);
                self.set_char(row - 1, col, character as u8, color);
            }
        }
        self.clear_row(24);
        self.column_position = 0;
    }
    pub fn delete_last(&mut self) {
        if self.column_position != 0 {
            self.column_position -= 1;
            self.set_char(24, self.column_position, b' ', Color16::White as u8);
        }
    }
}
impl fmt::Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s, Color16::White);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::writer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
static PRINT_SERIAL: bool = false;
static SERIAL_PORT: u16 = 0x3F8;
lazy_static! {
    static ref SERIAL: Mutex<SerialPort> = Mutex::new(unsafe {
        let mut port = SerialPort::new(SERIAL_PORT);
        port.init();
        port
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    interrupts::without_interrupts(|| {
        if PRINT_SERIAL {
            SERIAL.lock().write_fmt(args).unwrap();
        }
        WRITER.lock().write_fmt(args).unwrap();
    });
}
