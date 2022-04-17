use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy  {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8  {
        null_mut() //alloc error
    }


    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called!")   // do nothing
    }
}

#[global_allocator]
static ALLOCATOR: Dummy = Dummy; 

pub const HEAP_START: usize = 0x_4444_4444_0000; // error because virtual memory region is not mapped to phycical memory yet.
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB


use x86_64::{
    structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
    VirtAddr
};
