#![no_std]
extern crate alloc;

use alloc::boxed::Box;
pub use block::registry::*;
use core::ops::Deref;

pub mod block;

pub trait Device<Id: DeviceId> {
    fn id(&self) -> Id;
}

impl<Id> Device<Id> for Box<dyn Device<Id>>
where
    Id: DeviceId,
{
    fn id(&self) -> Id {
        self.deref().id()
    }
}

pub trait DeviceId: Copy + Eq {}
