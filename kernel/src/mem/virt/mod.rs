use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::Size4KiB;

pub use memory_backed::*;
pub use pm_object::*;
pub use vfs_node_backed::*;
pub use vm_object::*;

mod memory_backed;
mod pm_object;
mod vfs_node_backed;
mod vm_object;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AllocationError {
    OutOfMemory,
    PageAlreadyMapped,
    PageAlreadyMappedInHugePage,
    IoError,
}

impl From<MapToError<Size4KiB>> for AllocationError {
    fn from(value: MapToError<Size4KiB>) -> Self {
        match value {
            MapToError::FrameAllocationFailed => Self::OutOfMemory,
            MapToError::ParentEntryHugePage => Self::PageAlreadyMappedInHugePage,
            MapToError::PageAlreadyMapped(_) => Self::PageAlreadyMapped,
        }
    }
}
