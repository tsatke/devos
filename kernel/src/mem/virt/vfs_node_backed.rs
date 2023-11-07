use alloc::sync::Arc;
use core::slice::from_raw_parts_mut;

use spin::RwLock;
use x86_64::structures::paging::{Page, PageSize, Size4KiB};
use x86_64::VirtAddr;

use crate::io::vfs::{vfs, VfsNode};
use crate::mem::virt::{AllocationError, AllocationStrategy, MemoryBackedVmObject, VmObject};

#[derive(Debug)]
pub struct VfsNodeBackedVmObject {
    node: VfsNode,
    offset: usize,
    vm_object: MemoryBackedVmObject,
}

impl VfsNodeBackedVmObject {
    pub fn create(
        node: VfsNode,
        offset: usize,
        addr: VirtAddr,
        size: usize,
    ) -> Result<Self, AllocationError> {
        let fs = node.fs().clone();
        let mut guard = fs.write();
        let pm_object = guard.create_pm_object_for_mmap(node.handle())?;
        let should_map = pm_object.phys_frames().len() > 0;
        let allocation_strategy = pm_object.allocation_strategy();
        let mut vm_object = MemoryBackedVmObject::new(
            Arc::new(RwLock::new(pm_object)),
            allocation_strategy,
            addr,
            size,
        );
        if should_map {
            // the file system provided physical memory for the file, immediately map it
            vm_object.map_pages()?;
        }
        Ok(Self {
            node,
            offset,
            vm_object,
        })
    }
}

impl VmObject for VfsNodeBackedVmObject {
    fn addr(&self) -> VirtAddr {
        self.vm_object.addr()
    }

    fn size(&self) -> usize {
        self.vm_object.size()
    }

    fn allocation_strategy(&self) -> AllocationStrategy {
        self.vm_object.allocation_strategy()
    }

    fn prepare_for_access(&self, offset: usize) -> Result<(), AllocationError> {
        // make sure that the accessed page is already mapped
        self.vm_object.prepare_for_access(offset)?;

        // TODO: read from the vfsnode into the accessed page

        let accessed_page = Page::<Size4KiB>::containing_address(self.addr() + offset);
        let mut slice = unsafe {
            from_raw_parts_mut(
                accessed_page.start_address().as_mut_ptr::<u8>(),
                Size4KiB::SIZE as usize,
            )
        };
        vfs()
            .read(&self.node, &mut slice, self.offset + offset)
            .map_err(|_| AllocationError::IoError)?;

        Ok(())
    }
}

// we don't need to implement drop, because the drop impl for the underlying memory backed
// vm object and the drop impl for the vfs node should take care of everything
