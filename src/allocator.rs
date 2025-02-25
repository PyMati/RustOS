use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use fixed::FixedSizeBlockAllocator;
use list::LinkedListAllocator;
// use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub struct ExampleAllocator;
pub mod bump;
pub mod fixed;
pub mod list;

use bump::BmpAlloc;

pub const HEAP_START: usize = 0x_4444_4444_4444;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

// Example
unsafe impl GlobalAlloc for ExampleAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("Dealloc should be not called!")
    }
}

// #[global_allocator]
// static ALLOCATOR: LockedHeap = LockedHeap::empty();
// #[global_allocator]
// static ALLOCATOR: MutexWrapper<BmpAlloc> = MutexWrapper::new(BmpAlloc::new());
// #[global_allocator]
// static ALLOCATOR: MutexWrapper<LinkedListAllocator> = MutexWrapper::new(LinkedListAllocator::new());

#[global_allocator]
static ALLOCATOR: MutexWrapper<FixedSizeBlockAllocator> =
    MutexWrapper::new(FixedSizeBlockAllocator::new());

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let start = VirtAddr::new(HEAP_START as u64);
        let end = start + HEAP_SIZE - 1u64;
        let start_page = Page::containing_address(start);
        let end_page = Page::containing_address(end);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct MutexWrapper<A> {
    inner: spin::Mutex<A>,
}

impl<A> MutexWrapper<A> {
    pub const fn new(inner: A) -> Self {
        MutexWrapper {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr
    } else {
        addr - remainder + align
    }
}
