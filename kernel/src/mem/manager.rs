use bootloader_api::BootInfo;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

use crate::mem::physical::PhysicalFrameAllocator;

static mut MEMORY_MANAGER: Option<Mutex<MemoryManager>> = None;

pub(in crate::mem) fn init(boot_info: &'static BootInfo) {
    unsafe {
        let mm = MemoryManager {
            alloc: PhysicalFrameAllocator::from(&boot_info.memory_regions),
        };
        let _ = MEMORY_MANAGER.insert(Mutex::new(mm));
    }
}

pub struct MemoryManager {
    alloc: PhysicalFrameAllocator,
}

impl MemoryManager {
    pub fn lock() -> MutexGuard<'static, Self> {
        Self::ref_lock().lock()
    }

    pub fn is_locked() -> bool {
        Self::ref_lock().is_locked()
    }

    fn ref_lock() -> &'static Mutex<Self> {
        unsafe { MEMORY_MANAGER.as_ref() }.expect("memory manager not initialized yet")
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.alloc.allocate_frame()
    }
}

/// A frame allocator that delegates frame allocations to the [`MemoryManager`].
pub(in crate::mem) struct FrameAllocatorDelegate;

unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorDelegate {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        MemoryManager::lock().allocate_frame()
    }
}
