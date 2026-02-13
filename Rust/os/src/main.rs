#![no_std] // Don't link the Rust standard library
#![no_main] // Disable all Rust-level entry points

use bootloader_api::entry_point;
use core::panic::PanicInfo;

entry_point!(kernel_main);

/// Write a byte to the serial port (COM1 at I/O port 0x3F8)
fn serial_print_byte(byte: u8) {
    unsafe {
        // Wait until the transmit buffer is empty (bit 5 of LSR at port 0x3FD)
        while (x86_port_read(0x3FD) & 0x20) == 0 {}
        // Write the byte
        x86_port_write(0x3F8, byte);
    }
}

/// Print a byte string to serial port
fn serial_print(s: &[u8]) {
    for &byte in s {
        if byte == b'\n' {
            serial_print_byte(b'\r'); // serial needs \r\n
        }
        serial_print_byte(byte);
    }
}

/// Read from x86 I/O port
unsafe fn x86_port_read(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

/// Write to x86 I/O port
unsafe fn x86_port_write(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("al") value, in("dx") port, options(nomem, nostack));
}

fn kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    serial_print(b"Hello World!\n");

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
