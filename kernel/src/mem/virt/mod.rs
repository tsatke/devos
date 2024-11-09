use core::error::Error;
use derive_more::Display;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::Size4KiB;

pub use file_backed::*;
pub use manager::*;
pub use memory_backed::*;
pub use pm_object::*;
pub use vm_object::*;

mod file_backed;
pub mod heap;
mod manager;
mod memory_backed;
mod pm_object;
mod vm_object;

#[derive(Display, Debug, Copy, Clone, Eq, PartialEq)]
pub enum AllocationError {
    #[display("out of memory")]
    OutOfMemory,
    #[display("page already mapped")]
    PageAlreadyMapped,
    #[display("page already mapped in huge page")]
    PageAlreadyMappedInHugePage,
    #[display("io error")]
    IoError,
}

impl Error for AllocationError {}

impl From<MapToError<Size4KiB>> for AllocationError {
    fn from(value: MapToError<Size4KiB>) -> Self {
        match value {
            MapToError::FrameAllocationFailed => Self::OutOfMemory,
            MapToError::ParentEntryHugePage => Self::PageAlreadyMappedInHugePage,
            MapToError::PageAlreadyMapped(_) => Self::PageAlreadyMapped,
        }
    }
}
