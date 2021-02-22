#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga_buffer;

// this function is called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // ! return type here marks panic as diverging function https://doc.rust-lang.org/1.30.0/book/first-edition/functions.html#diverging-functions
    // i.e. it will never return
    loop {}
}

static HELLO: &[u8] = b"mini_os: Hello World!";
static BLANK: &[u8] = b"! ";

#[no_mangle] // prevent mangling function names
pub extern "C" fn _start() -> ! {
    // `0xb8000` is the address of the VGA buffer
    // VGA buffer is 25 lines of 80 chars, each one has an ASCII byte and a color byte
    let vga_buffer = 0xb8000 as *mut u8;
    // ^ this is a raw pointer https://doc.rust-lang.org/stable/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer

    /*
       for (i, &byte) in HELLO.iter().enumerate() {
           // unsafe only allows you 5 extra things
           // https://doc.rust-lang.org/stable/book/ch19-01-unsafe-rust.html#unsafe-superpowers
           unsafe {
               *vga_buffer.offset(i as isize * 2) = byte; // offset does pointer arithmetic, it's i * 2 from 0xb8000
    *vga_buffer.offset(i as isize * 2 + 1) = (11 as u8) << 4 | 5 as u8;
               // first 4 bits foreground color, next 3 background color, last bit for blinking
           }
       }
    */

    vga_buffer::print_something(" BOOM!");

    use core::fmt::Write;
    //lock here uses spinlock Mutex
    vga_buffer::WRITER
        .lock()
        .write_str("Hello again !\n")
        .unwrap();

    write!(
        vga_buffer::WRITER.lock(),
        "The numbers are {} and {}",
        42,
        1.0 / 3.0
    )
    .unwrap();

    /*
    for j in 0..24 {
        for (i, &byte) in HELLO.iter().enumerate() {
            // unsafe only allows you 5 extra things https://doc.rust-lang.org/stable/book/ch19-01-unsafe-rust.html#unsafe-superpowers
            unsafe {
                *vga_buffer.offset((80 * j + i as isize) * 2) = byte; // offset does pointer arithmetic, it's i * 2 from 0xb8000
                *vga_buffer.offset((80 * j + i as isize) * 2 + 1) =
                    (14 as u8) << 4 | (j % 15) as u8;
            }
        }
    }
    */
    loop {}
}
