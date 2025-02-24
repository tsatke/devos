use crate::limine::{HHDM_REQUEST, MEMORY_MAP_REQUEST};
use crate::mem::address_space::{recursive_index_to_virtual_address, RECURSIVE_INDEX};
use crate::mem::heap::Heap;
use conquer_once::spin::OnceCell;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use limine::memory_map::EntryType;
use spin::RwLock;
use virtual_memory_manager::{AlreadyReserved, Segment, VirtualMemoryManager};
use x86_64::VirtAddr;

static VMM: OnceCell<RwLock<VirtualMemoryManager>> = OnceCell::uninit();

fn vmm() -> &'static RwLock<VirtualMemoryManager> {
    VMM.get().expect("virtual memory should be initialized")
}

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
        VirtualMemory::mark_as_reserved(segment)
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
                    EntryType::KERNEL_AND_MODULES,
                    EntryType::BOOTLOADER_RECLAIMABLE,
                ]
                .contains(&e.entry_type)
            })
            .for_each(|e| {
                let segment = Segment::new(VirtAddr::new(e.base + hhdm_offset), e.length);
                VirtualMemory::mark_as_reserved(segment)
                    .expect("segment should not be reserved yet");
            });
    }

    // heap
    VirtualMemory::mark_as_reserved(Segment::new(Heap::bottom(), Heap::size() as u64))
        .expect("heap should not be reserved yet");
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnedSegment {
    inner: Segment,
}

impl OwnedSegment {
    #[allow(dead_code)]
    pub fn leak(self) -> Segment {
        ManuallyDrop::new(self).inner
    }
}

impl Drop for OwnedSegment {
    fn drop(&mut self) {
        VirtualMemory::release(self);
    }
}

impl Deref for OwnedSegment {
    type Target = Segment;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct VirtualMemory;

impl VirtualMemory {
    #[allow(dead_code)]
    pub fn reserve(n: usize) -> Option<OwnedSegment> {
        vmm()
            .write()
            .reserve(n)
            .map(|segment| OwnedSegment { inner: segment })
    }

    pub fn mark_as_reserved(segment: Segment) -> Result<(), AlreadyReserved> {
        vmm().write().mark_as_reserved(segment)
    }

    fn release(segment: &mut OwnedSegment) -> bool {
        vmm().write().release(segment.inner)
    }
}
