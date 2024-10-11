use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use conquer_once::spin::OnceCell;
use linked_list_allocator::LockedHeap;
use x86_64::VirtAddr;

use crate::mem::Size;
use crate::process::IN_RESCHEDULE;

pub static KERNEL_HEAP_ADDR: OnceCell<VirtAddr> = OnceCell::uninit();
pub static KERNEL_HEAP_LEN: Size = Size::MiB(8);

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// # Safety
/// This function must be called only once.
/// The caller must ensure that `heap_start` - `heap_start + heap_size is mapped and valid
/// for reads and writes.
pub unsafe fn init(heap_start: *mut u8, heap_size: usize) {
    ALLOCATOR.init(heap_start, heap_size);
    INITIALIZED.store(true, Relaxed);
}

pub fn heap_initialized() -> bool {
    INITIALIZED.load(Relaxed)
}

/// Returns how much free memory is left in the heap in bytes.
pub fn free() -> usize {
    ALLOCATOR.0.lock().free()
}

/// Returns the size of the heap in bytes.
pub fn size() -> usize {
    ALLOCATOR.0.lock().size()
}

/// Returns how much memory is used in the heap in bytes.
pub fn used() -> usize {
    ALLOCATOR.0.lock().used()
}

pub struct Allocator(LockedHeap);

impl Allocator {
    const fn new() -> Self {
        Self(LockedHeap::empty())
    }

    fn init(&self, heap_start: *mut u8, heap_size: usize) {
        unsafe {
            self.0.lock().init(heap_start, heap_size);
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    #[inline(always)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if IN_RESCHEDULE.load(Relaxed) {
            panic!("can't allocate memory while rescheduling");
        }
        unsafe { self.0.alloc(layout) }
    }

    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if IN_RESCHEDULE.load(Relaxed) {
            panic!("can't de-allocate memory while rescheduling");
        }
        unsafe { self.0.dealloc(ptr, layout) }
    }
}
