use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;
use limine::memory_map::{Entry, EntryType};
use x86_64::structures::paging::{FrameAllocator, Page, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub static STAGE1_ALLOCATED_FRAMES: AtomicUsize = AtomicUsize::new(0);

pub struct TrivialPhysicalFrameAllocator {
    regions: &'static [&'static Entry],
    next: usize,
}

unsafe impl Send for TrivialPhysicalFrameAllocator {}

impl TrivialPhysicalFrameAllocator {
    pub fn from(regions: &'static [&'static Entry]) -> Self {
        Self { regions, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.regions
            .iter()
            // get usable regions from memory map
            .filter(|r| r.entry_type == EntryType::USABLE)
            // map each region to its address range
            .map(|r| r.base..r.base + r.length)
            // transform to an iterator of frame start addresses
            .flat_map(|r| r.step_by(Page::<Size4KiB>::SIZE as usize))
            // create `PhysFrame` types from the start addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for TrivialPhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        STAGE1_ALLOCATED_FRAMES.fetch_add(1, Relaxed);
        frame
    }
}
