use bootloader_api::BootInfo;
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};

use crate::mem::physical::MemoryMapPhysicalFrameAllocator;
use crate::mem::physical::TrivialPhysicalFrameAllocator;

static MEMORY_MANAGER: Mutex<Option<PhysicalMemoryManager>> = Mutex::new(None);

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
    let _ = MEMORY_MANAGER.lock().insert(mm);
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
    pub fn allocate_frame() -> Option<PhysFrame> {
        MEMORY_MANAGER
            .lock()
            .as_mut()
            .map(|mm| mm.alloc.allocate_frame())
            .flatten()
    }

    pub fn deallocate_frame(frame: PhysFrame) {
        MEMORY_MANAGER.lock().as_mut().map(|mm| unsafe {
            mm.alloc.deallocate_frame(frame);
        });
    }
}

/// A frame allocator that delegates frame allocations to the [`PhysicalMemoryManager`].
pub(in crate::mem) struct FrameAllocatorDelegate;

unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorDelegate {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        PhysicalMemoryManager::allocate_frame()
    }
}
