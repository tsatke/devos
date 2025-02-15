use crate::mem::address_space::{virt_addr_from_page_table_indices, AddressSpace};
use crate::mem::phys::PhysicalMemory;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;
use log::info;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

static HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);
static HEAP_START: VirtAddr = virt_addr_from_page_table_indices([257, 0, 0, 0], 0);
static INITIAL_HEAP_SIZE: usize = 5 * 1024 * 1024; // 5 MiB

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

pub(in crate::mem) fn init(address_space: &AddressSpace) {
    assert!(PhysicalMemory::is_initialized());

    info!("initializing heap at {:p}", HEAP_START);
    let page_range = PageRangeInclusive {
        start: Page::<Size4KiB>::containing_address(HEAP_START),
        end: Page::<Size4KiB>::containing_address(HEAP_START + INITIAL_HEAP_SIZE as u64),
    };

    address_space
        .map_range(
            page_range,
            PhysicalMemory::allocate_frames_non_contiguous(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )
        .expect("should be able to map heap");

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START.as_mut_ptr(), INITIAL_HEAP_SIZE);
    }

    HEAP_INITIALIZED.store(true, Relaxed);
}

#[derive(Copy, Clone)]
pub struct Heap;

impl Heap {
    pub fn is_initialized() -> bool {
        HEAP_INITIALIZED.load(Relaxed)
    }

    pub fn free() -> usize {
        ALLOCATOR.lock().free()
    }

    pub fn used() -> usize {
        ALLOCATOR.lock().used()
    }

    pub fn size() -> usize {
        ALLOCATOR.lock().size()
    }

    pub fn bottom() -> VirtAddr {
        VirtAddr::new(ALLOCATOR.lock().bottom() as u64)
    }
}

impl core::fmt::Debug for Heap {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Heap")
            .field("initialized", &Self::is_initialized())
            .field("free", &Self::free())
            .field("used", &Self::used())
            .field("size", &Self::size())
            .finish()
    }
}
