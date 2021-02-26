#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use mini_os::println;

// this function is called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // ! return type here marks panic as diverging function
    // https://doc.rust-lang.org/1.30.0/book/first-edition/functions.html#diverging-functions
    // i.e. it will never return

    println!("{}", info);
    mini_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mini_os::test_panic_handler(info)
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // `0xb8000` is the address of the VGA buffer

    println!("mini_os: Hello {}!", 1000);

    mini_os::init();

    #[cfg(test)]
    test_main();
    use mini_os::memory;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::{Translate, Page};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    let page = Page::containing_address(VirtAddr::new(0xdeadbeef));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };



    let addresses = [
        //the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        //some stack page
        0x0100_0020_1a10,
        //virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];


    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    mini_os::hlt_loop();
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
