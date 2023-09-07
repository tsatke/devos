use crate::io::vfs::{
    CreateError, CreateNodeType, Dir, Inode, InodeBase, InodeNum, LookupError, MountError,
    Permission, Stat,
};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub struct Ext2Fs<T> {
    inner: InnerHandle<T>,
}

impl<T> Ext2Fs<T>
where
    T: filesystem::BlockDevice + Send + Sync + 'static,
{
    pub fn new(inner: ext2::Ext2Fs<T>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner { fs: inner })),
        }
    }

    pub fn root_inode(&self) -> Inode {
        Inode::new_dir(Ext2Dir {
            fs: self.inner.clone(),
        })
    }
}

type InnerHandle<T> = Arc<RwLock<Inner<T>>>;

struct Inner<T> {
    fs: ext2::Ext2Fs<T>,
}

pub struct Ext2Dir<T> {
    fs: InnerHandle<T>,
}

impl<T> InodeBase for Ext2Dir<T>
where
    T: filesystem::BlockDevice + Send + Sync,
{
    fn num(&self) -> InodeNum {
        1_u64.into()
    }

    fn name(&self) -> String {
        "hello".to_string()
    }

    fn stat(&self) -> Stat {
        Stat {
            ..Default::default()
        }
    }
}

impl<T> Dir for Ext2Dir<T>
where
    T: filesystem::BlockDevice + Send + Sync,
{
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<Inode, LookupError> {
        todo!()
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
        todo!()
    }

    fn mount(&mut self, node: Inode) -> Result<(), MountError> {
        todo!()
    }
}
