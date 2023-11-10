use alloc::string::String;
use core::slice::from_raw_parts_mut;

use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use crate::mem::virt::{AllocationError, AllocationStrategy, MemoryBackedVmObject, VmObject};
use crate::process;
use crate::process::fd::Fileno;

// FIXME: once we have write support in the fs, the drop impl should write dirty pages back to disk
#[derive(Debug)]
pub struct FileBackedVmObject {
    fd: Fileno,
    offset: usize,
    vm_object: MemoryBackedVmObject,
}

impl FileBackedVmObject {
    pub fn create(
        name: String,
        fd: Fileno,
        offset: usize,
        addr: VirtAddr,
        size: usize,
        flags: PageTableFlags,
    ) -> Result<Self, AllocationError> {
        Ok(Self {
            fd,
            offset,
            vm_object: MemoryBackedVmObject::create(
                name,
                addr,
                size,
                AllocationStrategy::AllocateOnAccess,
                flags,
            )?,
        })
    }
}

impl VmObject for FileBackedVmObject {
    fn name(&self) -> &str {
        self.vm_object.name()
    }

    fn addr(&self) -> VirtAddr {
        self.vm_object.addr()
    }

    fn size(&self) -> usize {
        self.vm_object.size()
    }

    fn flags(&self) -> PageTableFlags {
        self.vm_object.flags()
    }

    fn allocation_strategy(&self) -> AllocationStrategy {
        self.vm_object.allocation_strategy()
    }

    fn underlying_fd(&self) -> Option<Fileno> {
        Some(self.fd)
    }

    fn prepare_for_access(&self, offset: usize) -> Result<(), AllocationError> {
        let fd = self.fd;
        let file_offset = self.offset + offset;
        // make sure that the accessed page is already mapped
        self.vm_object
            .prepare_for_access_and_modify_page(offset, |page| {
                let slice = unsafe {
                    // safety: we just mapped the page, so we can safely zero it
                    from_raw_parts_mut(
                        page.start_address().as_mut_ptr::<u8>(),
                        page.size() as usize,
                    )
                };
                slice.fill(0);

                process::current()
                    .read_at(fd, slice, file_offset)
                    .map_err(|_| AllocationError::IoError)?;
                Ok(())
            })
    }
}
