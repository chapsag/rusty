use x86_64::{structures::paging::{PageTable, OffsetPageTable, PhysFrame, Size4KiB, FrameAllocator}, VirtAddr, PhysAddr};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

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

pub struct EmptyFrameAllocator;

// Responsable for allocating frames for new page table f the are needed by map_to
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap, // reference to the memory map
    next: usize, // keeps track of the next free frame
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self { // Caller must guarantee that memory map is valid
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> { // Iterator over usables frames in memory map
        let regions = self.memory_map.iter(); // get usable regions from memory map as an iterator
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable); // skip any reserved or unavailable region
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr()); // transform as an iterator of address ranges
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096)); // get start address of each frame and using step of 4096
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr))) // transform to iterator of frames
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next); // get next frame from iterator
        self.next += 1; // increment next
        frame // return next frame
    }
}
