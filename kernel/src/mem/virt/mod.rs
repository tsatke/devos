use crate::limine::{HHDM_REQUEST, MEMORY_MAP_REQUEST};
use crate::mem::address_space::{recursive_index_to_virtual_address, RECURSIVE_INDEX};
use crate::mem::heap::Heap;
use conquer_once::spin::OnceCell;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use limine::memory_map::EntryType;
use spin::RwLock;
use virtual_memory_manager::{AlreadyReserved, Segment, VirtualMemoryManager};
use x86_64::structures::paging::{PageSize, Size4KiB};
use x86_64::VirtAddr;

static VMM: OnceCell<RwLock<VirtualMemoryManager>> = OnceCell::uninit();

fn vmm() -> &'static RwLock<VirtualMemoryManager> {
    VMM.get().expect("virtual memory should be initialized")
}

#[allow(clippy::missing_panics_doc)]
pub fn init() {
    VMM.init_once(|| {
        RwLock::new(VirtualMemoryManager::new(
            VirtAddr::new(0xFFFF_8000_0000_0000),
            0x0000_7FFF_FFFF_FFFF,
        ))
    });

    // recursive mapping
    {
        let recursive_index = *RECURSIVE_INDEX
            .get()
            .expect("recursive index should be initialized");
        let vaddr = recursive_index_to_virtual_address(recursive_index);
        let len = 512 * 1024 * 1024 * 1024; // 512 GiB
        let segment = Segment::new(vaddr, len);
        VirtualMemoryHigherHalf::mark_as_reserved(segment)
            .expect("recursive index should not be reserved yet");
    }

    // kernel code and bootloader reclaimable
    {
        let hhdm_offset = HHDM_REQUEST.get_response().unwrap().offset();
        MEMORY_MAP_REQUEST
            .get_response()
            .unwrap()
            .entries()
            .iter()
            .filter(|e| {
                [
                    EntryType::EXECUTABLE_AND_MODULES,
                    EntryType::BOOTLOADER_RECLAIMABLE,
                ]
                .contains(&e.entry_type)
            })
            .for_each(|e| {
                let segment = Segment::new(VirtAddr::new(e.base + hhdm_offset), e.length);
                VirtualMemoryHigherHalf::mark_as_reserved(segment)
                    .expect("segment should not be reserved yet");
            });
    }

    // heap
    VirtualMemoryHigherHalf::mark_as_reserved(Segment::new(Heap::bottom(), Heap::size() as u64))
        .expect("heap should not be reserved yet");
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnedSegment {
    inner: Segment,
}

impl OwnedSegment {
    #[must_use]
    pub fn leak(self) -> Segment {
        ManuallyDrop::new(self).inner
    }
}

impl Drop for OwnedSegment {
    fn drop(&mut self) {
        VirtualMemoryHigherHalf::release_owned(self);
    }
}

impl Deref for OwnedSegment {
    type Target = Segment;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct VirtualMemoryHigherHalf;

impl VirtualMemoryHigherHalf {
    #[allow(dead_code)]
    #[must_use]
    pub fn reserve(pages: usize) -> Option<OwnedSegment> {
        vmm()
            .write()
            .reserve(pages * 4096)
            .map(|segment| OwnedSegment { inner: segment })
    }

    /// # Errors
    /// This function returns an error if the segment is already reserved.
    pub fn mark_as_reserved(segment: Segment) -> Result<(), AlreadyReserved> {
        debug_assert!(segment.start.is_aligned(Size4KiB::SIZE));
        debug_assert_eq!(segment.len % Size4KiB::SIZE, 0);

        vmm().write().mark_as_reserved(segment)
    }

    fn release_owned(segment: &mut OwnedSegment) -> bool {
        unsafe { Self::release(segment.inner) }
    }

    /// # Safety
    /// The caller must ensure that the segment is not used after releasing it,
    /// and that the segment was previously reserved by this virtual memory manager.
    #[must_use]
    pub unsafe fn release(segment: Segment) -> bool {
        vmm().write().release(segment)
    }
}
