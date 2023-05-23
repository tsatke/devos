use bootloader_api::BootInfo;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};

use crate::mem::physical_stage1::TrivialPhysicalFrameAllocator;
use crate::mem::physical_stage2::MemoryMapPhysicalFrameAllocator;

static mut MEMORY_MANAGER: Option<Mutex<MemoryManager>> = None;

// TODO: remove the two-stage approach
// Currently, we need this, because the address space requires an initialized stage1 allocator
// to allocate page table frames.
pub(in crate::mem) fn init_stage1(boot_info: &'static BootInfo) {
    init(Allocator::Stage1(TrivialPhysicalFrameAllocator::from(
        &boot_info.memory_regions,
    )));
}

pub(in crate::mem) fn init_stage2(boot_info: &'static BootInfo) {
    init(Allocator::Stage2(MemoryMapPhysicalFrameAllocator::from(
        &boot_info.memory_regions,
    )));
}

fn init(stage: Allocator) {
    let mm = MemoryManager { alloc: stage };
    unsafe {
        let _ = MEMORY_MANAGER.insert(Mutex::new(mm));
    }
}

enum Allocator {
    Stage1(TrivialPhysicalFrameAllocator),
    Stage2(MemoryMapPhysicalFrameAllocator),
}

impl Allocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        match self {
            Allocator::Stage1(alloc) => alloc.allocate_frame(),
            Allocator::Stage2(alloc) => alloc.allocate_frame(),
        }
    }

    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        unsafe {
            match self {
                Allocator::Stage1(_) => {}
                Allocator::Stage2(alloc) => alloc.deallocate_frame(frame),
            }
        }
    }
}

pub struct MemoryManager {
    alloc: Allocator,
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

    pub fn deallocate_frame(&mut self, frame: PhysFrame) {
        unsafe { self.alloc.deallocate_frame(frame) }
    }
}

/// A frame allocator that delegates frame allocations to the [`MemoryManager`].
pub(in crate::mem) struct FrameAllocatorDelegate;

unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorDelegate {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        MemoryManager::lock().allocate_frame()
    }
}
