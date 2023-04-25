use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{FrameAllocator, Page, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub struct PhysicalFrameAllocator {
    regions: &'static MemoryRegions,
    next: usize,
}

impl PhysicalFrameAllocator {
    pub unsafe fn from(regions: &'static MemoryRegions) -> Self {
        Self { regions, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.regions
            .iter()
            // get usable regions from memory map
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            // map each region to its address range
            .map(|r| r.start..r.end)
            // transform to an iterator of frame start addresses
            .flat_map(|r| r.step_by(Page::<Size4KiB>::SIZE as usize))
            // create `PhysFrame` types from the start addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
