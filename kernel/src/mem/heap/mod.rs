use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init(heap_start: *mut u8, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
        INITIALIZED.store(true, Relaxed);
    }
}

pub fn heap_initialized() -> bool {
    unsafe { INITIALIZED.load(Relaxed) }
}

/// Returns how much free memory is left in the heap in bytes.
pub fn free() -> usize {
    ALLOCATOR.lock().free()
}

/// Returns the size of the heap in bytes.
pub fn size() -> usize {
    ALLOCATOR.lock().size()
}

/// Returns how much memory is used in the heap in bytes.
pub fn used() -> usize {
    ALLOCATOR.lock().used()
}
