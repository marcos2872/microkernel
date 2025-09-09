#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod vga_buffer;
mod interrupts;

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen!();
    println!("Welcome to the microkernel!");
    println!("This is a basic implementation in Rust.");

    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    println!("It did not crash!");
    loop {}
}
