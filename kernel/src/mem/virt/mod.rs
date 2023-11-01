use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::Size4KiB;

pub use pm_object::*;
pub use vm_object::*;

mod pm_object;
mod vm_object;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AllocationError {
    OutOfMemory,
    PageAlreadyMapped,
    PageAlreadyMappedInHugePage,
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