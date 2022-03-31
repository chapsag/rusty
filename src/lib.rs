#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // Unsafe because we are not sure if the PICS is initialized.
    x86_64::instructions::interrupts::enable();
}


use core::panic::PanicInfo;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    shutdown(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    shutdown(QemuExitCode::Failed);
    hlt_loop();
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}


// Let CPU uses less power
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)] // 4 bytes
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn shutdown(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe { // writing to I/O == arbitrary behavior
        let mut port = Port::new(0xf4); // isa-debug-exit iobase port
        port.write(exit_code as u32);
    }
}

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;