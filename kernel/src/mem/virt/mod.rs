pub use pm_object::*;
pub use vm_object::*;

mod pm_object;
mod vm_object;

#[derive(Debug)]
pub struct AllocationError;
