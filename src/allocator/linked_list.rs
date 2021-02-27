use super::{align_up, Locked};
use core::{mem, ptr};
use alloc::alloc::{GlobalAlloc, Layout};


struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>, // &'static mut -  semantically describes an owned object behind a pointer.
}

impl ListNode {
    const fn new(size: usize) -> ListNode {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize // Self here is alias for ListNode
        // *const Self meaning a pointer to ListNode? https://doc.rust-lang.org/std/primitive.pointer.html
        // Self being the type of self, self:Self  https://stackoverflow.com/questions/32304595/whats-the-difference-between-self-and-self
        // also https://doc.rust-lang.org/beta/std/keyword.self.html
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }

}


pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self { // Self here is aliast for LinkedListAllocator
        Self {
            head: ListNode::new(0)
        }
    }


    // Initialise the allocator with heap bounds
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        //make sure that the free region is aligned and has enough space to contain ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        let mut new_head = ListNode::new(size);
        new_head.next = self.head.next.take(); // take() clears current head.next to None, and sets new_head.next to 2nd Node
        let new_head_ptr = addr as *mut ListNode; // addr as pointer to ListNode
        new_head_ptr.write(new_head); // write new_head ListNode to memory under  addr

        self.head.next = Some(&mut *new_head_ptr); // head.next -> new_head

        //todo!();
    }

    //Looks for a free region with a given size and alignment and removes it from the list
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        //current node
        let mut current = &mut self.head;

        //iterate through the list, while there are tail Nodes
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                // this region will do for allocation, remove it from the list
                let next_next = region.next.take(); // clear it's tail
                let res = Some((current.next.take().unwrap(), alloc_start));
                current.next = next_next; // from A -> B -> C to  A -> C
                return res;
            } else {
                //region won't work, move to next in the list
                current = current.next.as_mut().unwrap()
            }
        }
        //no region found
        None
    }


    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?; // force Option
        if alloc_end > region.end_addr() {
            // region is too small
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            //we can't use it since leftover memory region is too small to fit ListNode
            // so we can't add it to FreeList and it will never be reused again!
            return Err(());
        }

        Ok(alloc_start)
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// region is also capable of storing a `ListNode`.
    ///
    /// Returns the adjusted size and alignment as a (size, align) tuple.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout.align_to(mem::align_of::<ListNode>())
            .expect("[LinkedListAllocator.size_align] adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}


unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // layout adjustments
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("allow overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }

    }

    // ToDo maybe implement merging freed blocks to prevent fragmentation ?
    // Instead of inserting freed memory blocks at the beginning of the linked list on deallocate, it always keeps the list sorted by start address. This way, merging can be performed directly on the deallocate call by examining the addresses and sizes of the two neighbor blocks in the list.
    // sample implementation here https://github.com/phil-opp/linked-list-allocator/blob/251eb1b0728e9b9cdd1dd6281effa0a6c6d70d63/src/lib.rs#L117
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        //perform layout adjustments
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size);
    }
}
