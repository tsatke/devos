use core::fmt::Debug;

use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use crate::mem::virt::{AllocationError, AllocationStrategy};

pub trait VmObject: Debug + Send + Sync {
    fn name(&self) -> &str;

    fn addr(&self) -> VirtAddr;

    fn size(&self) -> usize;

    fn flags(&self) -> PageTableFlags;

    fn allocation_strategy(&self) -> AllocationStrategy;

    fn contains_addr(&self, addr: VirtAddr) -> bool {
        let my_addr = self.addr();
        let my_size = self.size();
        addr >= my_addr && addr < my_addr + my_size
    }

    fn prepare_for_access(&self, offset: usize) -> Result<(), AllocationError>;
}
