use alloc::sync::Arc;
use core::slice;

use derive_more::Constructor;
use spin::RwLock;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

use crate::mem::physical::PhysicalMemoryManager;
use crate::mem::virt::{AllocationError, AllocationStrategy, PmObject, VmObject};
use crate::{map_page, process};

#[derive(Constructor, Debug)]
pub struct MemoryBackedVmObject {
    underlying: Arc<RwLock<PmObject>>,
    allocation_strategy: AllocationStrategy,
    addr: VirtAddr,
    size: usize,
}

impl MemoryBackedVmObject {
    pub fn create(
        addr: VirtAddr,
        size: usize,
        allocation_strategy: AllocationStrategy,
    ) -> Result<Self, AllocationError> {
        let pm_object = PmObject::create(size, allocation_strategy)?;
        let mut res = Self::new(
            Arc::new(RwLock::new(pm_object)),
            allocation_strategy,
            addr,
            size,
        );

        if allocation_strategy == AllocationStrategy::AllocateNow {
            // we should also map the memory immediately
            res.map_pages()?;

            unsafe {
                // safety: we just mapped the pages, so we can safely zero them
                slice::from_raw_parts_mut(addr.as_mut_ptr::<u8>(), size).fill(0);
            }
        }

        Ok(res)
    }

    pub(in crate::mem::virt) fn map_pages(&mut self) -> Result<(), AllocationError> {
        let first_page = Page::<Size4KiB>::containing_address(self.addr);
        let last_page = Page::<Size4KiB>::containing_address(self.addr + self.size);
        let page_range = Page::<Size4KiB>::range(first_page, last_page);
        let current_process = process::current();
        let mut address_space = current_process.address_space().write();
        let guard = self.underlying.read();
        let frames = guard.phys_frames();
        for (page, frame) in page_range.zip(frames.iter().cloned()) {
            unsafe {
                address_space
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    )?
                    .flush();
            }
        }
        Ok(())
    }
}

impl VmObject for MemoryBackedVmObject {
    fn addr(&self) -> VirtAddr {
        self.addr
    }

    fn size(&self) -> usize {
        self.size
    }

    fn allocation_strategy(&self) -> AllocationStrategy {
        self.allocation_strategy
    }

    fn prepare_for_access(&self, offset: usize) -> Result<(), AllocationError> {
        let page = Page::<Size4KiB>::containing_address(self.addr + offset);
        let frame = PhysicalMemoryManager::lock().allocate_frame().unwrap();
        self.underlying.write().add_phys_frame(frame);
        map_page!(
            page,
            frame,
            Size4KiB,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        );
        unsafe {
            // safety: we just mapped the page, so we can safely zero it
            slice::from_raw_parts_mut(
                page.start_address().as_mut_ptr::<u8>(),
                page.size() as usize,
            )
            .fill(0);
        }
        Ok(())
    }
}

impl Drop for MemoryBackedVmObject {
    fn drop(&mut self) {
        assert!(
            interrupts::are_enabled(),
            "interrupts must be enabled when dropping a vmobject"
        );
        deallocate(self);
    }
}

fn deallocate(vm_object: &MemoryBackedVmObject) {
    let current_process = process::current();
    let mut address_space = current_process.address_space().write();
    let range = Page::<Size4KiB>::range_inclusive(
        Page::<Size4KiB>::containing_address(vm_object.addr),
        Page::<Size4KiB>::containing_address(
            vm_object.addr + vm_object.size.wrapping_add_signed(-1),
        ),
    );
    for page in range {
        if let Ok((_, flusher)) = address_space.unmap(page) {
            flusher.flush(); // we might not have mapped all pages
        }
    }
}
