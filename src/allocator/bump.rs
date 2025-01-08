use super::{align_up, MutexWrapper};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct BmpAlloc {
    start: usize,
    end: usize,
    next: usize,
    allocations: usize,
}

impl BmpAlloc {
    pub const fn new() -> Self {
        BmpAlloc {
            start: 0,
            end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.start = heap_start;
        self.end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for MutexWrapper<BmpAlloc> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.end {
            ptr::null_mut()
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.start;
        }
    }
}
