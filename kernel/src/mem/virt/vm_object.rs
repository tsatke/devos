use alloc::slice;
use alloc::sync::Arc;

use spin::RwLock;
use x86_64::instructions::interrupts;
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
        Self::create(Arc::new(RwLock::new(pm_object)), addr, size)
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

    fn create(
        pm_object: Arc<RwLock<PmObject>>,
        addr: VirtAddr,
        size: usize,
    ) -> Result<Self, AllocationError> {
        let allocation_strategy = pm_object.read().allocation_strategy();

        if allocation_strategy == AllocationStrategy::AllocateNow {
            // we should also map the memory immediately
            let first_page = Page::<Size4KiB>::containing_address(addr);
            let last_page = Page::<Size4KiB>::containing_address(addr + size);
            let page_range = Page::<Size4KiB>::range(first_page, last_page);
            let current_process = process::current();
            let address_space = current_process.address_space().read();
            let pm_object_guard = pm_object.read();
            let frames = pm_object_guard.phys_frames();
            for (page, frame) in page_range.zip(frames.iter().cloned()) {
                unsafe {
                    address_space
                        .map_to(
                            page,
                            frame,
                            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        )?
                        .flush(); // TODO: can we do one big flush at the end?
                }
            }

            unsafe {
                // safety: we just mapped the pages, so we can safely zero them
                slice::from_raw_parts_mut(addr.as_mut_ptr::<u8>(), size).fill(0);
            }
        }

        Ok(Self {
            underlying: pm_object,
            addr,
            size,
        })
    }

    pub fn addr(&self) -> &VirtAddr {
        &self.addr
    }

    pub fn contains_addr(&self, addr: VirtAddr) -> bool {
        addr >= self.addr && addr < self.addr + self.size
    }
}

impl Drop for VmObject {
    fn drop(&mut self) {
        assert!(
            interrupts::are_enabled(),
            "interrupts must be enabled when dropping a vmobject"
        );
        deallocate_vm_object(self);
    }
}

fn deallocate_vm_object(vm_object: &VmObject) {
    let current_process = process::current();
    let address_space = current_process.address_space().read();
    for page in Page::<Size4KiB>::range(
        Page::<Size4KiB>::containing_address(vm_object.addr),
        Page::<Size4KiB>::containing_address(vm_object.addr + vm_object.size), // no -1 since the range is exclusive
    ) {
        address_space
            .unmap(page)
            .unwrap() // if we allocated correctly, this can't happen
            .1
            .flush();
    }
}
