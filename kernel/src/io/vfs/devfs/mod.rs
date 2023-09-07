use crate::io::vfs::{Fs, Inode};
use alloc::string::ToString;

mod chardev;
mod dir;

pub use chardev::*;
pub use dir::*;

pub struct DevFs {
    root: Inode,
}

impl Fs for DevFs {
    fn root_inode(&self) -> Inode {
        self.root.clone()
    }
}

impl DevFs {
    pub fn new(fsid: u64) -> Self {
        Self {
            root: Inode::new_dir(DevFsDir::new(fsid, "/".to_string())),
        }
    }
}
