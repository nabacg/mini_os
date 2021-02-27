#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use mini_os::println;
extern crate alloc;


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

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // `0xb8000` is the address of the VGA buffer

    println!("mini_os: Hello {}!", 1000);

    mini_os::init();

    #[cfg(test)]
    test_main();
    use mini_os::memory;
    use mini_os::allocator;
    use x86_64::VirtAddr;



    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap allocation failed");
    let x = Box::new(32);
    println!("heap_value at {:p}", x); // {:p} pointer formatting https://doc.rust-lang.org/core/fmt/trait.Pointer.html

    let mut vec:Vec<u32> = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1,2,3]);
    let cloned_reference = reference_counted.clone();
    println!("current referece count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("current referece count is {}", Rc::strong_count(&cloned_reference));
/*


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
    */

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
