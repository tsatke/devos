use core::slice::from_raw_parts_mut;

use derive_more::Constructor;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use crate::io::vfs::{vfs, VfsNode};
use crate::mem::virt::{AllocationError, MemoryBackedVmObject, VmObject};

// FIXME: once we have write support in the fs, the drop impl should write dirty pages back to disk
#[derive(Debug, Constructor)]
pub struct FileBackedVmObject {
    node: VfsNode,
    offset: usize,
    underlying: MemoryBackedVmObject,
}

impl VmObject for FileBackedVmObject {
    fn name(&self) -> &str {
        self.underlying.name()
    }

    fn addr(&self) -> VirtAddr {
        self.underlying.addr()
    }

    fn size(&self) -> usize {
        self.underlying.size()
    }

    fn flags(&self) -> PageTableFlags {
        self.underlying.flags()
    }

    fn underlying_node(&self) -> Option<&VfsNode> {
        Some(&self.node)
    }

    fn prepare_for_access(&self, offset: usize) -> Result<(), AllocationError> {
        let file_offset = self.offset + offset;
        // make sure that the accessed page is already mapped
        self.underlying
            .prepare_for_access_and_modify_page(offset, |page| {
                let slice = unsafe {
                    // safety: we just mapped the page, so we can safely zero it
                    from_raw_parts_mut(
                        page.start_address().as_mut_ptr::<u8>(),
                        page.size() as usize,
                    )
                };
                slice.fill(0);

                vfs()
                    .read(&self.node, slice, file_offset)
                    .map_err(|_| AllocationError::IoError)?;
                Ok(())
            })
    }
}
