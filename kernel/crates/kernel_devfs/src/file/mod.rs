use kernel_vfs::{ReadError, Stat, StatError, WriteError};

mod block;
pub use block::*;
mod null;
pub use null::*;
mod serial;
pub use serial::*;
mod zero;
pub use zero::*;

pub trait DevFile: Send + Sync {
    fn read(&mut self, buf: &mut [u8], offset: usize) -> Result<usize, ReadError>;
    fn write(&mut self, buf: &[u8], offset: usize) -> Result<usize, WriteError>;
    fn stat(&mut self, stat: &mut Stat) -> Result<(), StatError>;
}
