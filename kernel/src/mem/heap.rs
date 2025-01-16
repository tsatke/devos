use crate::mem::phys::PhysicalMemory;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

static HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub(in crate::mem) fn init() {
    assert!(PhysicalMemory::is_initialized());

    todo!();

    HEAP_INITIALIZED.store(true, Relaxed);
}

#[derive(Copy, Clone)]
pub struct Heap;

impl Heap {
    pub fn is_initialized() -> bool {
        HEAP_INITIALIZED.load(Relaxed)
    }
}
