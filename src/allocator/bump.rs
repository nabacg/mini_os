use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    // Creates a new empty bump allocator
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// initializes the allocator with given heap (start, size) bounds
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut balloc = self.lock();
        let alloc_start = align_up(balloc.next, layout.align());

        // checked_add will prevent int overflow https://doc.rust-lang.org/std/primitive.usize.html#method.checked_add
        let end_addr_maybe = alloc_start.checked_add(layout.size());
        let alloc_end = match end_addr_maybe {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > balloc.heap_end {
            ptr::null_mut() // OOM exception
        } else {
            balloc.next = alloc_end;
            balloc.allocations += 1;
            balloc.next as *mut u8
        }
    }


    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut balloc = self.lock();
        balloc.allocations -= 1;
        if balloc.next ==  (ptr as usize) + layout.size() {
            balloc.next -= (ptr as usize) - layout.size();
        }

        if balloc.allocations == 0 {
            balloc.next = balloc.heap_start;
        };
    }
}
