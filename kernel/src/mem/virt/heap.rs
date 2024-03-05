use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use conquer_once::spin::OnceCell;
use linked_list_allocator::LockedHeap;
use x86_64::VirtAddr;

use crate::mem::Size;

pub static KERNEL_HEAP_ADDR: OnceCell<VirtAddr> = OnceCell::uninit();
pub static KERNEL_HEAP_LEN: Size = Size::MiB(8);

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

/// # Safety
/// This function must be called only once.
/// The caller must ensure that `heap_start` - `heap_start + heap_size is mapped and valid
/// for reads and writes.
pub unsafe fn init(heap_start: *mut u8, heap_size: usize) {
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
