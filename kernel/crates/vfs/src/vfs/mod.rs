use crate::fs::FileSystem;
use crate::node::VfsNode;
use crate::path::{OwnedPath, Path};
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::RwLock;

pub use error::*;

mod error;
pub mod node;

#[cfg(test)]
pub mod testing;

pub struct Vfs {
    file_systems: BTreeMap<OwnedPath, Arc<RwLock<dyn FileSystem>>>, // TODO: maybe a trie would be better here?
}

impl Vfs {
    pub const fn new() -> Self {
        Self {
            file_systems: BTreeMap::new(),
        }
    }

    pub fn mount<P, F>(&mut self, mount_point: P, fs: F) -> Result<(), MountError>
    where
        P: AsRef<Path>,
        F: FileSystem + 'static,
    {
        let mount_point = mount_point.as_ref().to_owned();
        if self.file_systems.contains_key(&mount_point) {
            return Err(MountError::AlreadyMounted);
        }

        // TODO: check whether the mount_point is a directory

        self.file_systems
            .insert(mount_point, Arc::new(RwLock::new(fs)));
        Ok(())
    }

    pub fn unmount<P>(&mut self, mount_point: P) -> Result<(), UnmountError>
    where
        P: AsRef<Path>,
    {
        let owned = mount_point.as_ref().to_owned();
        self.file_systems
            .remove(&owned)
            .map(|_| ())
            .ok_or(UnmountError::NotMounted)
    }

    pub fn open<P>(&self, path: P) -> Result<VfsNode, OpenError>
    where
        P: AsRef<Path>,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::TestFs;
    use crate::Vfs;
    use alloc::vec;

    #[test]
    fn test_mount() {
        let mut fs = TestFs::default();
        fs.insert_file("/foo/bar.txt", vec![0xAA; 25]);

        let mut vfs = Vfs::new();
        vfs.mount("/root", fs).unwrap();
        assert!(vfs.mount("/root", TestFs::default()).is_err());
    }
}
