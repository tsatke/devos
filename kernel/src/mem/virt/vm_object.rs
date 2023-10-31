use alloc::sync::Arc;

use spin::RwLock;
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

use crate::mem::physical::PhysicalMemoryManager;
use crate::mem::virt::PmObject;
use crate::mem::virt::{AllocationError, AllocationStrategy};
use crate::{map_page, process};

#[derive(Debug)]
pub struct VmObject {
    underlying: Arc<RwLock<PmObject>>,
    addr: VirtAddr,
    size: usize,
}

impl VmObject {
    pub fn create_memory_backed(
        addr: VirtAddr,
        size: usize,
        allocation_strategy: AllocationStrategy,
    ) -> Result<Self, AllocationError> {
        let pm_object = PmObject::create_memory_backed(size, allocation_strategy)?;
        Ok(Self::create(Arc::new(RwLock::new(pm_object)), addr, size))
    }

    pub fn prepare_for_access(&self, offset: usize) -> Result<(), AllocationError> {
        let page = Page::<Size4KiB>::containing_address(self.addr + offset);
        let frame = PhysicalMemoryManager::lock().allocate_frame().unwrap();
        self.underlying.write().add_phys_frame(frame);
        map_page!(
            page,
            frame,
            Size4KiB,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        );
        Ok(())
    }

    fn create(pm_object: Arc<RwLock<PmObject>>, addr: VirtAddr, size: usize) -> Self {
        let allocation_strategy = pm_object.read().allocation_strategy();

        if allocation_strategy == AllocationStrategy::AllocateNow {
            // we should also map the memory immediately
            let first_page = Page::<Size4KiB>::containing_address(addr);
            let last_page = Page::<Size4KiB>::containing_address(VirtAddr::new(
                first_page.start_address().as_u64() + size as u64 - 1,
            ));
            let page_range = first_page..last_page;
            let current_process = process::current();
            let current_process_data = current_process.read();
            let address_space = current_process_data.address_space();
            let pm_object_guard = pm_object.read();
            let frames = pm_object_guard.phys_frames();
            for (page, frame) in page_range.zip(frames.iter().cloned()) {
                unsafe {
                    address_space
                        .map_to(
                            page,
                            frame,
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        )
                        .unwrap()
                        .flush(); // TODO: can we do one big flush at the end?
                }
            }
        }

        Self {
            underlying: pm_object,
            addr,
            size,
        }
    }

    pub fn addr(&self) -> &VirtAddr {
        &self.addr
    }

    pub fn contains_addr(&self, addr: VirtAddr) -> bool {
        addr >= self.addr && addr < self.addr + self.size
    }
}
