/* 

Fixed-Size Block Allocator
--------------------------

Round up the requested allocation size to the next block size. For example, when an allocation of 12 bytes is requested, we would choose the block size 16 in the above example.
Retrieve the head pointer for the list, e.g. from an array. For block size 16, we need to use head_16.
Remove the first block from the list and return it.

TODO: Try implementing a slab allocator.
*/

struct ListNode  {
    next: Option<&'static mut ListNode>
}

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct FixedSizeBlockAllocator  {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()], // array of head pointers
    fallback_allocator: linked_list_allocator::Heap, // fallback allocator, if no block is available
}

impl FixedSizeBlockAllocator  {
    pub const fn new() -> Self {
        //Init the list heads with empty nodes 
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }
}

use alloc::alloc::Layout;
use core::ptr;

impl FixedSizeBlockAllocator {
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

// Allocate a block of memory of the given size required by the given layout.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

use super::Locked;
use core::{mem, ptr::NonNull};
use alloc::alloc::GlobalAlloc;

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout:Layout) -> *mut u8  {
        let mut allocator = self.lock(); // get a mutable reference to the allocator
        match list_index(&layout) { // calculate the appropriate block size for the given layout
            Some(index) => {
               match allocator.list_heads[index].take() {
                   Some(node) => {
                       allocator.list_heads[index] = node.next.take(); // Try to remove the first block from the list
                       node as *mut ListNode as *mut u8
                   },
                   None => {  // Need to construct a new block
                       let block_size = BLOCK_SIZES[index]; // No block exists in list, allocate a new block
                       let block_align = block_size; // Align the block to the block size (power of 2)
                       let layout = Layout::from_size_align(block_size, block_align).unwrap();
                       allocator.fallback_alloc(layout)
                   }
               }
            }
            None => { // no block available, allocate from the fallback allocator
                allocator.fallback_alloc(layout)
            }
        }
        
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock(); // get a mutable reference to the allocator
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]); // Make sure the block size is large enough to hold the ListNode
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]); // Make sure the block size is aligned to the ListNode
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node); // write the new node to the list
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => { // Not created by our implementation, give it back to the fallback allocator 
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
    
}