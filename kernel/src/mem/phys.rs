use crate::mem::heap::Heap;
use conquer_once::spin::OnceCell;
use core::mem::swap;
use limine::memory_map::{Entry, EntryType};
use spin::Mutex;
use x86_64::structures::paging::frame::PhysFrameRangeInclusive;
use x86_64::structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

static PHYS_ALLOC: OnceCell<Mutex<MultiStageAllocator>> = OnceCell::uninit();

fn allocator() -> &'static Mutex<MultiStageAllocator> {
    PHYS_ALLOC
        .get()
        .expect("physical allocator not initialized")
}

#[derive(Copy, Clone)]
pub struct PhysicalMemory;

impl PhysicalMemory {
    pub fn is_initialized() -> bool {
        PHYS_ALLOC.is_initialized()
    }

    pub fn allocate_frame() -> Option<PhysFrame> {
        allocator().lock().allocate_frame()
    }

    pub fn allocate_frames(n: usize) -> Option<PhysFrameRangeInclusive> {
        allocator().lock().allocate_frames(n)
    }

    pub fn deallocate_frame(frame: PhysFrame) {
        allocator().lock().deallocate_frame(frame)
    }

    pub fn deallocate_frames(range: PhysFrameRangeInclusive) {
        allocator().lock().deallocate_frames(range)
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalMemory {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        Self::allocate_frame()
    }
}

/// Initialize the first stage of physical memory management: a simple bump
/// allocator.
pub(in crate::mem) fn init_stage1(entries: &'static [&'static Entry]) {
    let stage1 = MultiStageAllocator::Stage1(PhysicalBumpAllocator::new(entries));
    PHYS_ALLOC.init_once(|| Mutex::new(stage1));
}

pub(in crate::mem) fn init_stage2() {
    let mut guard = allocator().lock();

    let stage1 = match &*guard {
        MultiStageAllocator::Stage1(a) => a,
        _ => unreachable!(),
    };
    let bitmap_allocator = PhysicalBitmapAllocator::create_from_stage1(stage1);
    let mut stage2 = MultiStageAllocator::Stage2(bitmap_allocator);
    swap(&mut *guard, &mut stage2);
}

pub trait PhysicalFrameAllocator {
    /// Allocates a single physical frame. If there is no more physical memory,
    /// this function returns `None`.
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_frames(1).map(|range| range.start)
    }

    /// Allocates `n` contiguous physical frames. If there is no more physical
    /// memory, this function returns `None`.
    fn allocate_frames(&mut self, n: usize) -> Option<PhysFrameRangeInclusive>;

    /// Deallocates a single physical frame.
    ///
    /// # Panics
    /// If built with `debug_assertions`, this function panics if the frame is
    /// already deallocated or not allocated yet.
    fn deallocate_frame(&mut self, frame: PhysFrame);

    /// Deallocates a range of physical frames.
    ///
    /// # Panics
    /// If built with `debug_assertions`, this function panics if any frame in
    /// the range is already deallocated or not allocated yet.
    /// Deallocation of remaining frames will not be attempted.
    fn deallocate_frames(&mut self, range: PhysFrameRangeInclusive) {
        for frame in range {
            self.deallocate_frame(frame);
        }
    }
}

enum MultiStageAllocator {
    Stage1(PhysicalBumpAllocator),
    Stage2(PhysicalBitmapAllocator),
}

impl PhysicalFrameAllocator for MultiStageAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        match self {
            Self::Stage1(a) => a.allocate_frame(),
            Self::Stage2(a) => a.allocate_frame(),
        }
    }

    fn allocate_frames(&mut self, n: usize) -> Option<PhysFrameRangeInclusive> {
        match self {
            Self::Stage1(a) => a.allocate_frames(n),
            Self::Stage2(a) => a.allocate_frames(n),
        }
    }

    fn deallocate_frame(&mut self, frame: PhysFrame) {
        match self {
            Self::Stage1(a) => a.deallocate_frame(frame),
            Self::Stage2(a) => a.deallocate_frame(frame),
        }
    }

    fn deallocate_frames(&mut self, range: PhysFrameRangeInclusive) {
        match self {
            Self::Stage1(a) => a.deallocate_frames(range),
            Self::Stage2(a) => a.deallocate_frames(range),
        }
    }
}

struct PhysicalBumpAllocator {
    regions: &'static [&'static Entry],
    next_frame: usize,
}

impl PhysicalBumpAllocator {
    fn new(regions: &'static [&'static Entry]) -> Self {
        Self {
            regions,
            next_frame: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.regions
            .iter()
            .filter(|region| region.entry_type == EntryType::USABLE)
            .map(|region| region.base..region.length)
            .flat_map(|r| r.step_by(Size4KiB::SIZE as usize))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

impl PhysicalFrameAllocator for PhysicalBumpAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next_frame);
        if frame.is_some() {
            self.next_frame += 1;
        }
        frame
    }

    fn allocate_frames(&mut self, n: usize) -> Option<PhysFrameRangeInclusive> {
        unimplemented!(
            "the stage1 physical frame allocator doesn't support allocation of contiguous frames"
        )
    }

    fn deallocate_frame(&mut self, frame: PhysFrame) {
        todo!()
    }
}

struct PhysicalBitmapAllocator {}

impl PhysicalBitmapAllocator {
    fn create_from_stage1(stage1: &PhysicalBumpAllocator) -> Self {
        assert!(Heap::is_initialized());
        todo!()
    }
}

impl PhysicalFrameAllocator for PhysicalBitmapAllocator {
    fn allocate_frames(&mut self, n: usize) -> Option<PhysFrameRangeInclusive> {
        todo!()
    }

    fn deallocate_frame(&mut self, frame: PhysFrame) {
        todo!()
    }
}
