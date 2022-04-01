use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use pic8259::ChainedPics; // Represent Secondary and Primary PICs
use spin::Mutex; // Spinlock
use lazy_static::lazy_static;
use crate::{println, print, gdt, hlt_loop};


// Pics:(Programmable interupt controller) are used to handle interrupts. Range from 32 to 47.
// Enable asynchrounous interrupts. Avoid deathlock with VGA_BUFFER.
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// Enable safe mutable access to the PICs thanks to Mutex
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}


lazy_static! { // Use unsafe behind the scene.
    static ref IDT: InterruptDescriptorTable =  {
        let mut idt = InterruptDescriptorTable::new(); // mute for modify breakpoints entry
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler); // Because of IndexMut can access with indexing syntax.
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_idt()  {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", _stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()); // Send End of Interrupt signal to PICs
    }
}

extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode)  {
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read()); // Read the address that caused the page fault
    println!("Error Code: {:?}", _error_code);
    println!("{:#?}", _stack_frame);
    hlt_loop(); // Halt the CPU because of the page fault
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame)  {
    
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use x86_64::instructions::port::Port;

    lazy_static! { // Create a static keyboard US instance protected by Mutex
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock(); // Lock the keyboard
    let mut port = Port::new(0x60); // I/O port for keyboard

    let scancode: u8 = unsafe { port.read() }; // Read the scancode from the keyboard

    // Decode the scancode
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) { // If the scancode is valid
        if let Some(key) = keyboard.process_keyevent(key_event) { // check press event and decode the key
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8()); // Send End of Interrupt signal to PICs
    }   
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame, _error_code: u64) -> ! // diverging cant return from double fault
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", _stack_frame);
}


#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}