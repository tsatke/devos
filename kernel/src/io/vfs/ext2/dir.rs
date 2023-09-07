use crate::io::vfs;
use crate::io::vfs::ext2::ext2_inode_to_inode;
use crate::io::vfs::{
    CreateError, CreateNodeType, Dir, Inode, InodeBase, LookupError, Permission, Stat,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct Ext2Dir<T> {
    inner: vfs::ext2::InnerHandle<T>,
    dir: ext2::Directory,
    name: String,
}

impl<T> Ext2Dir<T> {
    pub(crate) fn new(
        inner: vfs::ext2::InnerHandle<T>,
        dir: ext2::Directory,
        name: String,
    ) -> Self {
        Self { inner, dir, name }
    }
}

impl<T> InodeBase for Ext2Dir<T>
where
    T: filesystem::BlockDevice + Send + Sync,
{
    fn name(&self) -> String {
        self.name.clone() // TODO: remove clone, but that requires some fixing in other places, where the name is behind a lock
    }

    fn stat(&self) -> Stat {
        Stat {
            dev: self.inner.read().fsid,
            inode: self.dir.inode_address().get().into(),
            ..Default::default()
        }
    }
}

impl<T> Dir for Ext2Dir<T>
where
    T: filesystem::BlockDevice + 'static + Send + Sync,
{
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<Inode, LookupError> {
        self.list_entry_inodes()?
            .find(|i| i.name() == name.as_ref())
            .ok_or(LookupError::NoSuchEntry)
    }

    fn create(
        &mut self,
        name: &dyn AsRef<str>,
        typ: CreateNodeType,
        permission: Permission,
    ) -> Result<Inode, CreateError> {
        todo!()
    }

    fn children(&self) -> Result<Vec<Inode>, LookupError> {
        Ok(self.list_entry_inodes()?.collect())
    }
}

impl<T> Ext2Dir<T>
where
    T: 'static + Send + Sync + filesystem::BlockDevice,
{
    fn list_entry_inodes(&self) -> Result<impl Iterator<Item = Inode> + '_, LookupError> {
        Ok(self
            .inner
            .read()
            .fs
            .list_dir(&self.dir)
            .map_err(|_| LookupError::NoSuchEntry)?
            .into_iter()
            .map(|e| {
                (
                    e.name().map(|s| s.to_string()),
                    self.inner.read().fs.resolve_dir_entry(e).unwrap(),
                )
            })
            .map(|(name, inode)| {
                ext2_inode_to_inode(self.inner.clone(), inode, name.unwrap().to_string())
            }))
    }
}
