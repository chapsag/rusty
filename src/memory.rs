use x86_64::{structures::paging::{PageTable, OffsetPageTable, Page, PhysFrame, Mapper, Size4KiB, FrameAllocator}, VirtAddr, PhysAddr};


//Init offsetPageTable
// Unsafe because caller must guarantee that the comlete physical memory is mapped.
// return instance with static lifetime: valid for complete runtimr of the kernel.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static>  { 
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}


// Return a mutable reference to the active lvl4 table.
// Fn can only be called once to avoid aliasing mut refs.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable  {
    use x86_64::registers::control::Cr3;

    let (level_4_table_name_frame, _) = Cr3::read();
    let phys = level_4_table_name_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64(); // Get virtual address where table is mapped.
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr(); // Convert to mutable pointer.

    &mut *page_table_ptr // Return a mutable reference to the active lvl4 table.
}
