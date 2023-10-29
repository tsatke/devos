use bootloader_api::BootInfo;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};

use crate::mem::physical::MemoryMapPhysicalFrameAllocator;
use crate::mem::physical::TrivialPhysicalFrameAllocator;

static mut MEMORY_MANAGER: Option<Mutex<PhysicalMemoryManager>> = None;

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
    let mm = PhysicalMemoryManager { alloc: stage };
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

pub struct PhysicalMemoryManager {
    alloc: Allocator,
}

impl PhysicalMemoryManager {
    pub fn lock() -> MutexGuard<'static, Self> {
        Self::ref_lock().lock()
    }

    fn ref_lock() -> &'static Mutex<Self> {
        unsafe { MEMORY_MANAGER.as_ref() }.expect("memory manager not initialized yet")
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.alloc.allocate_frame()
    }

    #[allow(unused)] // TODO: remove this, we will use this eventually
    pub fn deallocate_frame(&mut self, frame: PhysFrame) {
        unsafe { self.alloc.deallocate_frame(frame) }
    }
}

/// A frame allocator that delegates frame allocations to the [`PhysicalMemoryManager`].
pub(in crate::mem) struct FrameAllocatorDelegate;

unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorDelegate {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        PhysicalMemoryManager::lock().allocate_frame()
    }
}
