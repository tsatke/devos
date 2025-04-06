use crate::{Device, DeviceId};
use alloc::boxed::Box;
use core::error::Error;
use core::ops::{Deref, DerefMut};

pub mod registry;

#[repr(transparent)]
pub struct BlockBuf<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> Deref for BlockBuf<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<const N: usize> DerefMut for BlockBuf<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[allow(clippy::missing_errors_doc)] // TODO: add documentation
pub trait BlockDevice<Id: DeviceId, const N: usize = 512>: Device<Id> {
    fn read_block(&mut self, block_num: usize, buf: &mut BlockBuf<N>)
    -> Result<(), Box<dyn Error>>;

    fn write_block(&mut self, block_num: usize, buf: &BlockBuf<N>) -> Result<(), Box<dyn Error>>;

    fn flush(&mut self) -> Result<(), Box<dyn Error>>;
}

impl<Id, const N: usize> Device<Id> for Box<dyn BlockDevice<Id, N>>
where
    Id: DeviceId,
{
    fn id(&self) -> Id {
        self.deref().id()
    }
}

impl<Id, const N: usize> BlockDevice<Id, N> for Box<dyn BlockDevice<Id, N>>
where
    Id: DeviceId,
{
    fn read_block(
        &mut self,
        block_num: usize,
        buf: &mut BlockBuf<N>,
    ) -> Result<(), Box<dyn Error>> {
        self.deref_mut().read_block(block_num, buf)
    }

    fn write_block(&mut self, block_num: usize, buf: &BlockBuf<N>) -> Result<(), Box<dyn Error>> {
        self.deref_mut().write_block(block_num, buf)
    }

    fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        self.deref_mut().flush()
    }
}
