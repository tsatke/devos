use crate::mem::physical::STAGE1_ALLOCATED_FRAMES;
use crate::mem::{is_heap_initialized, HEAP_SIZE};
use crate::serial_println;
use alloc::vec;
use alloc::vec::Vec;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use core::sync::atomic::Ordering::Relaxed;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum FrameState {
    Free,
    Allocated,
    NotUsable,
}

pub struct MemoryMapPhysicalFrameAllocator {
    frames: Vec<FrameState>,
    first_free: Option<usize>,
}

impl MemoryMapPhysicalFrameAllocator {
    pub fn from(regions: &'static MemoryRegions) -> Self {
        assert!(
            is_heap_initialized(),
            "Heap must first be initialized before using the physical memory map"
        );

        let total_mem_size = regions.iter().map(|r| r.end - r.start).sum::<u64>();
        serial_println!(
            "~{} MiB total physical memory available",
            (total_mem_size / 1024 / 1024) + 1
        );
        let frame_count = (total_mem_size / Size4KiB::SIZE) as usize;

        let mut frames = vec![FrameState::NotUsable; frame_count];

        serial_println!(
            "memory manager stage 1 allocated {} physical frames, {} of which belong to the kernel heap",
            unsafe { STAGE1_ALLOCATED_FRAMES.load(Relaxed) },
            HEAP_SIZE.bytes() / Size4KiB::SIZE as usize
        );

        // mark the usable frames as 'free'
        regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .map(|r| r.start..r.end)
            .flat_map(|r| r.step_by(Size4KiB::SIZE as usize))
            .map(|addr| PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr)))
            .for_each(|frame| {
                let frame_index = (frame.start_address().as_u64() / Size4KiB::SIZE) as usize;
                frames[frame_index] = FrameState::Free;
            });

        // mark the already allocated heap as 'allocated'
        let stage1_allocated_frames = unsafe { STAGE1_ALLOCATED_FRAMES.load(Relaxed) };
        regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .map(|r| r.start..r.end)
            .flat_map(|r| r.step_by(Size4KiB::SIZE as usize))
            .map(|addr| PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr)))
            // only the first HEAP_SIZE/PAGE_SIZE frames are already allocated
            .take(stage1_allocated_frames)
            .for_each(|frame| {
                let frame_index = (frame.start_address().as_u64() / Size4KiB::SIZE) as usize;
                frames[frame_index] = FrameState::Allocated;
            });

        Self {
            frames,
            first_free: Some(stage1_allocated_frames),
        }
    }

    fn frame_index_to_address(&self, index: usize) -> PhysAddr {
        PhysAddr::new(index as u64 * Size4KiB::SIZE)
    }

    fn frame_address_to_index(&self, addr: PhysAddr) -> usize {
        (addr.as_u64() / Size4KiB::SIZE) as usize
    }
}

unsafe impl FrameAllocator<Size4KiB> for MemoryMapPhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let first_free = self.first_free.unwrap_or(0);
        let index = self
            .frames
            .iter()
            .enumerate()
            .skip(first_free)
            .find(|(_, state)| matches!(state, FrameState::Free))?
            .0;
        self.frames[index] = FrameState::Allocated;
        self.first_free = self
            .frames
            .iter()
            .enumerate()
            .skip(index)
            .find(|(_, state)| matches!(state, FrameState::Free))
            .map(|(i, _)| i);
        Some(PhysFrame::from_start_address(self.frame_index_to_address(index)).unwrap())
    }
}

impl FrameDeallocator<Size4KiB> for MemoryMapPhysicalFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let index = self.frame_address_to_index(frame.start_address());
        if let Some(first_free) = self.first_free {
            if index < first_free {
                self.first_free = Some(index);
            }
        } else {
            self.first_free = Some(index);
        }
        self.frames[index] = FrameState::Free;
    }
}
