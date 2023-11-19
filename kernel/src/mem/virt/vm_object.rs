use core::fmt::Debug;
use core::slice::{from_raw_parts, from_raw_parts_mut};

use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use crate::io::vfs::VfsNode;
use crate::mem::virt::AllocationError;

pub trait VmObject: Debug + Send + Sync {
    fn name(&self) -> &str;

    fn addr(&self) -> VirtAddr;

    fn size(&self) -> usize;

    fn flags(&self) -> PageTableFlags;

    fn underlying_node(&self) -> Option<&VfsNode> {
        None
    }

    fn contains_addr(&self, addr: VirtAddr) -> bool {
        let my_addr = self.addr();
        let my_size = self.size();
        addr >= my_addr && addr < my_addr + my_size
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts(self.addr().as_ptr::<u8>(), self.size()) }
    }

    /// Creates a mutable slice from this vm object.
    ///
    /// # Safety
    /// The caller must ensure that there are no other references to the memory
    /// of this vm object, such as slices created by [`as_slice`], this method or other
    /// ways.
    #[allow(clippy::mut_from_ref)]
    unsafe fn as_slice_mut(&self) -> &mut [u8] {
        unsafe { from_raw_parts_mut(self.addr().as_mut_ptr::<u8>(), self.size()) }
    }

    fn prepare_for_access(&self, offset: usize) -> Result<(), AllocationError>;
}
