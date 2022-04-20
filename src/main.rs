#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rusty_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rusty_os::println;
extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

entry_point!(kernel_main); // macro to define the entry point of the program to avoid arbritary args


pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rusty_os::memory::{self, BootInfoFrameAllocator};
    use rusty_os::allocator;
    use x86_64::{VirtAddr};
    
    println!(" > Booting rusty, welcome MR. GOFFI");
    rusty_os::init();
    println!(" > Kernel init done");


    // crete missing page tables:
        // 1. Allocate unused frame from the passed frame_allocator
        // 2. Zero the frame to create a new, empty page table
        // 3. Map the entry  of the higher level table to that frame
        // 4. Contine with the next table level
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map)};    


    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));


    
    #[cfg(test)]
    test_main();

    rusty_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rusty_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rusty_os::test_panic_handler(info)
}