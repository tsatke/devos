use crate::mem::heap::Heap;
use alloc::vec;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use core::iter::from_fn;
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

#[allow(dead_code)]
impl PhysicalMemory {
    pub fn is_initialized() -> bool {
        PHYS_ALLOC.is_initialized()
    }

    pub fn allocate_frames_non_contiguous() -> impl Iterator<Item = PhysFrame> {
        from_fn(Self::allocate_frame)
    }

    /// Calls [`PhysicalFrameAllocator::allocate_frame`] on the current physical allocator.
    pub fn allocate_frame() -> Option<PhysFrame> {
        allocator().lock().allocate_frame()
    }

    /// Calls [`PhysicalFrameAllocator::allocate_frames`] on the current physical allocator.
    pub fn allocate_frames(n: usize) -> Option<PhysFrameRangeInclusive> {
        allocator().lock().allocate_frames(n)
    }

    /// Calls [`PhysicalFrameAllocator::deallocate_frame`] on the current physical allocator.
    pub fn deallocate_frame(frame: PhysFrame) {
        allocator().lock().deallocate_frame(frame);
    }

    /// Calls [`PhysicalFrameAllocator::deallocate_frames`] on the current physical allocator.
    pub fn deallocate_frames(range: PhysFrameRangeInclusive) {
        allocator().lock().deallocate_frames(range);
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

/// Initialize the second stage of physical memory management: a bitmap allocator.
/// This allocator requires that the heap is initialized and that stage1 was previously
/// initialized.
pub(in crate::mem) fn init_stage2() {
    let mut guard = allocator().lock();

    let MultiStageAllocator::Stage1(stage1) = &*guard else {
        unreachable!()
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
            .flat_map(|r| r.step_by(usize::try_from(Size4KiB::SIZE).expect("usize overflow")))
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

    fn allocate_frames(&mut self, _: usize) -> Option<PhysFrameRangeInclusive> {
        unimplemented!(
            "the stage1 physical frame allocator doesn't support allocation of contiguous frames"
        )
    }

    fn deallocate_frame(&mut self, _: PhysFrame) {
        unimplemented!("the stage1 physical frame allocator doesn't support deallocation of frames")
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum FrameState {
    Free,
    Allocated,
    NotUsable,
}

struct PhysicalBitmapAllocator {
    frames: Vec<FrameState>,
    first_free: Option<usize>,
}

impl PhysicalBitmapAllocator {
    fn create_from_stage1(stage1: &PhysicalBumpAllocator) -> Self {
        assert!(Heap::is_initialized());

        let regions = stage1.regions;
        let stage_one_next_free = stage1.next_frame;

        // TODO: we only need to consider all regions until the last one that is usable
        let highest_usable_address = regions.iter().fold(0, |highest_address, r| {
            if r.entry_type == EntryType::USABLE {
                r.base + r.length
            } else {
                highest_address
            }
        });

        let frame_count = (highest_usable_address / Size4KiB::SIZE) as usize;

        let mut frames = vec![FrameState::NotUsable; frame_count];

        regions
            .iter()
            .filter(|r| r.entry_type == EntryType::USABLE)
            .map(|r| r.base..r.base + r.length)
            .flat_map(|r| r.step_by(usize::try_from(Size4KiB::SIZE).expect("usize overflow")))
            .map(PhysAddr::new)
            .map(Self::frame_address_to_index)
            .enumerate()
            .for_each(|(i, frame)| {
                if i < stage_one_next_free {
                    frames[frame] = FrameState::Allocated;
                } else {
                    frames[frame] = FrameState::Free;
                }
            });

        Self {
            frames,
            first_free: Some(stage_one_next_free),
        }
    }

    fn frame_index_to_address(index: usize) -> PhysAddr {
        PhysAddr::new(index as u64 * Size4KiB::SIZE)
    }

    fn frame_address_to_index(addr: PhysAddr) -> usize {
        (addr.as_u64() / Size4KiB::SIZE) as usize
    }
}

impl PhysicalFrameAllocator for PhysicalBitmapAllocator {
    fn allocate_frames(&mut self, n: usize) -> Option<PhysFrameRangeInclusive> {
        let start_index = self
            .frames
            .windows(n)
            .skip(self.first_free.unwrap_or(0))
            .position(|window| window.iter().all(|&state| state == FrameState::Free))?;
        let end_index = start_index + n - 1;
        self.frames[start_index..=end_index]
            .iter_mut()
            .for_each(|state| *state = FrameState::Allocated);
        self.first_free = self
            .frames
            .iter()
            .skip(end_index)
            .position(|&state| state == FrameState::Free);
        Some(PhysFrameRangeInclusive {
            start: PhysFrame::containing_address(Self::frame_index_to_address(start_index)),
            end: PhysFrame::containing_address(Self::frame_index_to_address(end_index)),
        })
    }

    fn deallocate_frame(&mut self, frame: PhysFrame) {
        let index = Self::frame_address_to_index(frame.start_address());
        if self.first_free.is_some_and(|v| index < v) {
            self.first_free = Some(index);
        }

        debug_assert_eq!(self.frames[index], FrameState::Allocated);
        self.frames[index] = FrameState::Free;
    }
}
