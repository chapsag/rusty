use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0; // double fault stack

lazy_static! { // No init at compile time.
    static ref TSS: TaskStateSegment =  { // Task State Segment is a data structure that describes the state of a task.
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 5 pages
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // Arch64. Grow downwards.

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK }); // Act as memory for now. Unsafe race condition
            let stack_end = stack_start + STACK_SIZE;

            stack_end
        };
        tss
    };
}

lazy_static!  {
    static ref GDT: (GlobalDescriptorTable, Selectors) =  { // GlobalDescriptorTable is a data structure that describes the state of a GDT.
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment()); // Reload code segment to point to new kernel code segment.
        let tss_selector  = gdt.add_entry(Descriptor::tss_segment(&TSS)); // Load TSS segment to point to new TSS segment.
        (gdt, Selectors { code_selector, tss_selector }) // Return valide GDT and Selectors.
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.0.load();

    unsafe { // We can brake memory safety by loading invalid selectors 
        CS::set_reg(GDT.1.code_selector); // Reload code segment to point to new kernel code segment.
        load_tss(GDT.1.tss_selector); // Load TSS segment to point to new TSS segment.
    }
}