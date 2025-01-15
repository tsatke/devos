use crate::mem::physical_stage1::TrivialPhysicalFrameAllocator;
use limine::memory_map::Entry;
use spin::Mutex;
use x86_64::structures::paging::Size4KiB;
use x86_64::structures::paging::{FrameAllocator, PhysFrame};

static MEMORY_MANAGER: Mutex<Option<PhysicalMemoryManager>> = Mutex::new(None);

pub(in crate::mem) fn init_stage1(entries: &'static [&'static Entry]) {
    init(Allocator::Stage1(TrivialPhysicalFrameAllocator::from(
        entries,
    )));
}

pub(in crate::mem) fn init_stage2(_entries: &'static [&'static Entry]) {
    todo!()
}

fn init(stage: Allocator) {
    let mm = PhysicalMemoryManager { alloc: stage };
    let _ = MEMORY_MANAGER.lock().insert(mm);
}

enum Allocator {
    Stage1(TrivialPhysicalFrameAllocator),
}

impl Allocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        match self {
            Allocator::Stage1(alloc) => alloc.allocate_frame(),
        }
    }

    unsafe fn deallocate_frame(&mut self, _frame: PhysFrame) {
        match self {
            Allocator::Stage1(_) => {}
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
            .and_then(|mm| mm.alloc.allocate_frame())
    }

    pub fn deallocate_frame(frame: PhysFrame) {
        if let Some(mm) = MEMORY_MANAGER.lock().as_mut() {
            unsafe { mm.alloc.deallocate_frame(frame) };
        }
    }
}

/// A frame allocator that delegates frame allocations to the [`PhysicalMemoryManager`].
pub(in crate::mem) struct FrameAllocatorDelegate;

unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorDelegate {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        PhysicalMemoryManager::allocate_frame()
    }
}
