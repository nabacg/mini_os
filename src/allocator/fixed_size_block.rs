//use super::linked_list::LinkedListAllocator; // for when Free block merging is ready
use alloc::alloc::Layout;
use core::{mem, ptr, ptr::NonNull};

struct ListNode {
    next : Option<&'static mut ListNode>,
}


// block sizes to use

// because block sizes are powers of two they can also be used as block alignments (which have to be powers of two)
// this simplifies things
// also, don't define any block sizes smaller than 8 because each block must be capable of storing a 64-bit pointer to the next block when freed!
const BLOCK_SIZES:  &[usize] = &[8, 16,  32, 64, 128, 256, 512, 1024, 2048];


pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
    //fallback_allocator: LinkedListAllocator, // for when Free block merging is ready
}


impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()], // list of Nones for heads ;)
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }


    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    // allocate using fallback_allocator in case size too large for any of our fixed size blocks
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}


fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align()); // so if we have size 5, with a 8 byte align, minimum Layout is multiple of align, so it has to be at least 8.
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}



use super::Locked;
use alloc::alloc::GlobalAlloc;

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(head_index) => {
                match allocator.list_heads[head_index].take() {
                    Some(free_block) => {
                        allocator.list_heads[head_index] = free_block.next.take(); // "pointer magic", popping current free block from top of the list and pointing head to next element
                        free_block as *mut ListNode as *mut u8
                    }
                    None => {
                        // there is no block in the list for this block size, we need to allocate new list
                        let block_size = BLOCK_SIZES[head_index];
                        // below only works if all sizes are the power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        // once
                        allocator.fallback_alloc(layout)
                    }
                }
            },
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(head_index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[head_index].take(), // pointer magic, we're making a new node and point it at current head. Essentially push new_node to head of the list
                };
                //verify that block has the right size and alignment
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[head_index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[head_index]); // this is where we need sizes to be power of 2 for alignment which has to be power of 2
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[head_index] = Some(&mut *new_node_ptr); // write new node as head
            },
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }

}
