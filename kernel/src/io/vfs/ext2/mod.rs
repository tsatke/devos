use crate::io::vfs::Inode;
use alloc::string::String;
use alloc::sync::Arc;
use dir::Ext2Dir;
use ext2::{Directory, RegularFile};
use spin::RwLock;

mod dir;
mod file;

pub use dir::*;
pub use file::*;

pub struct Ext2Fs<T> {
    inner: InnerHandle<T>,
    root_node_name: String,
}

impl<T> Ext2Fs<T>
where
    T: filesystem::BlockDevice + Send + Sync + 'static,
{
    pub fn new(inner: ext2::Ext2Fs<T>, root_node_name: String) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner { fs: inner })),
            root_node_name,
        }
    }

    pub fn root_inode(&self) -> Option<Inode> {
        let dir = self.inner.read().fs.read_root_inode().ok()?;
        Some(Inode::new_dir(Ext2Dir::new(
            self.inner.clone(),
            dir,
            self.root_node_name.clone(),
        )))
    }
}

pub(crate) type InnerHandle<T> = Arc<RwLock<Inner<T>>>;

pub(crate) struct Inner<T> {
    fs: ext2::Ext2Fs<T>,
}

fn ext2_inode_to_inode<T>(inner: InnerHandle<T>, ext2_inode: ext2::Inode, name: String) -> Inode
where
    T: filesystem::BlockDevice + 'static + Send + Sync,
{
    match ext2_inode.typ() {
        ext2::Type::Directory => Inode::new_dir(Ext2Dir::new(
            inner,
            Directory::try_from(ext2_inode).unwrap(),
            name,
        )),
        ext2::Type::RegularFile => Inode::new_file(Ext2File::new(
            inner,
            RegularFile::try_from(ext2_inode).unwrap(),
            name,
        )),
        _ => todo!("todo: {:?}", ext2_inode.typ()),
    }
}
