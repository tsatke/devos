use alloc::string::String;

use crate::mem::address_space::AddressSpace;
pub use id::*;

mod id;

pub struct Process {
    pid: ProcessId,
    name: String,

    address_space: AddressSpace,
}
