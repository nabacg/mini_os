#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use mini_os::println;

// this function is called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // ! return type here marks panic as diverging function https://doc.rust-lang.org/1.30.0/book/first-edition/functions.html#diverging-functions
    // i.e. it will never return

    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mini_os::test_panic_handler(info)
}

static HELLO: &[u8] = b"mini_os: Hello World!";
static BLANK: &[u8] = b"! ";

#[no_mangle] // prevent mangling function names
pub extern "C" fn _start() -> ! {
    // `0xb8000` is the address of the VGA buffer

    println!("Hello {}!", 1000);

    #[cfg(test)]
    test_main();

    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}
