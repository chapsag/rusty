#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rusty_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rusty_os::println;
extern crate alloc;
use alloc::boxed::Box;

entry_point!(kernel_main); // macro to define the entry point of the program to avoid arbritary args


pub fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rusty_os::{memory, memory::BootInfoFrameAllocator};
    use x86_64::{VirtAddr};
    
    println!(" > Booting rusty, welcome MR. GOFFI");
    rusty_os::init();
    println!(" > Kernel init done");



    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map)};    
    // crete missing page tables:
        // 1. Allocate unused frame from the passed frame_allocator
        // 2. Zero the frame to create a new, empty page table
        // 3. Map the entry  of the higher level table to that frame
        // 4. Contine with the next table level

    // map unused memory

    let x = Box::new(42);


    
    #[cfg(test)]
    test_main();

    rusty_os::hlt_loop();
}


// This function is called on panic.
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