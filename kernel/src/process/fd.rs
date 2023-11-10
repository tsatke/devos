use core::sync::atomic::{AtomicUsize, Ordering};

use derive_more::{Constructor, Display};

use crate::io::vfs::{vfs, VfsError, VfsNode};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Display, Constructor)]
pub struct Fileno(usize);

impl Fileno {
    pub fn as_usize(self) -> usize {
        self.into()
    }
}

impl From<Fileno> for usize {
    fn from(value: Fileno) -> Self {
        value.0
    }
}

#[derive(Default, Debug)]
pub struct FilenoAllocator {
    inner: AtomicUsize,
}

impl FilenoAllocator {
    pub const fn new() -> Self {
        Self {
            inner: AtomicUsize::new(0),
        }
    }

    pub fn next(&self) -> Fileno {
        Fileno::new(self.inner.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug)]
pub struct FileDescriptor {
    node: VfsNode,
    offset: usize,
}

impl FileDescriptor {
    pub fn new(node: VfsNode) -> Self {
        Self { node, offset: 0 }
    }

    pub fn into_node(self) -> VfsNode {
        self.node
    }

    pub fn node(&self) -> &VfsNode {
        &self.node
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, VfsError> {
        match self.read_at(buf, self.offset) {
            Ok(v) => {
                self.offset += v;
                Ok(v)
            }
            e => e,
        }
    }

    pub fn read_at(&mut self, buf: &mut [u8], offset: usize) -> Result<usize, VfsError> {
        vfs().read(&self.node, buf, offset)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, VfsError> {
        match self.write_at(buf, self.offset) {
            Ok(v) => {
                self.offset += v;
                Ok(v)
            }
            e => e,
        }
    }

    pub fn write_at(&mut self, buf: &[u8], offset: usize) -> Result<usize, VfsError> {
        vfs().write(&self.node, buf, offset)
    }
}
