use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

/// Read from x86 I/O port
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

/// Write to x86 I/O port
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("al") value, in("dx") port, options(nomem, nostack));
}

pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    pub const fn new(base: u16) -> Self {
        SerialPort { base }
    }

    pub fn init(&self) {
        unsafe {
            outb(self.base + 1, 0x00); // Disable interrupts
            outb(self.base + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(self.base + 0, 0x01); // Set divisor to 1 (115200 baud)
            outb(self.base + 1, 0x00); //   (hi byte)
            outb(self.base + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(self.base + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
            outb(self.base + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    fn is_transmit_empty(&self) -> bool {
        unsafe { inb(self.base + 5) & 0x20 != 0 }
    }

    pub fn write_byte(&self, byte: u8) {
        while !self.is_transmit_empty() {}
        unsafe { outb(self.base, byte) }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let serial = SerialPort::new(0x3F8);
        serial.init();
        Mutex::new(serial)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("serial print failed");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
