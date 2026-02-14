#![no_std] // Don't link the Rust standard library
#![no_main] // Disable all Rust-level entry points

mod serial;

use bootloader_api::entry_point;
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    println!("Hello World!");
    println!("Numbers: {} {}", 42, 1.337);
    panic!("Some panic message");
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {}
}
