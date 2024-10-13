use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use ext2::Type;
use filesystem::BlockDevice;
use spin::RwLock;

use file::Ext2Inode;
use kernel_api::syscall::Stat;

use crate::io::path::{Component, Path};
use crate::io::vfs::error::{Result, VfsError};
use crate::io::vfs::{DirEntry, FileSystem, FileType, FsId, VfsHandle};

mod file;

static HANDLE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_handle() -> VfsHandle {
    VfsHandle::new(HANDLE_COUNTER.fetch_add(1, Relaxed))
}

pub struct VirtualExt2Fs<T> {
    fsid: FsId,
    handles: BTreeMap<VfsHandle, Arc<RwLock<Ext2Inode<T>>>>, // there might be multiple VfsHandles pointing to the same inode
    inner: Arc<RwLock<ext2::Ext2Fs<T>>>,
}

impl<T> VirtualExt2Fs<T>
where
    T: BlockDevice,
{
    pub fn new(fsid: FsId, inner: ext2::Ext2Fs<T>) -> Self {
        Self {
            fsid,
            handles: BTreeMap::new(),
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    fn resolve_handle(&self, handle: VfsHandle) -> Result<&Arc<RwLock<Ext2Inode<T>>>> {
        self.handles.get(&handle).ok_or(VfsError::HandleClosed)
    }

    fn find_inode(&self, path: &Path) -> Result<(ext2::InodeAddress, ext2::Inode)> {
        let root_inode = self
            .inner
            .read()
            .read_root_inode()
            .map_err(|_| VfsError::NoSuchFile)?;
        self.find_inode_from(path, root_inode.into_inner())
    }

    fn find_inode_from(
        &self,
        path: &Path,
        starting_point: (ext2::InodeAddress, ext2::Inode),
    ) -> Result<(ext2::InodeAddress, ext2::Inode)> {
        let path = path.to_owned();
        let components = path.components();
        let fs = self.inner.read();

        let (mut current_num, mut current) = starting_point;
        for component in components {
            match component {
                Component::RootDir => {}
                Component::CurrentDir => {} // do nothing,
                Component::ParentDir => {
                    todo!("parent dir");
                }
                Component::Normal(v) => {
                    let x = current.typ();
                    if x != Type::Directory {
                        // TODO: symlink support
                        return Err(VfsError::NoSuchFile);
                    }
                    // x is a directory
                    let found_entry = fs
                        .list_dir(&current) // list entries in the directory
                        .map_err(|_| VfsError::NoSuchFile)?
                        .into_iter()
                        .find(|entry| entry.name() == Some(v))
                        .ok_or(VfsError::NoSuchFile)?;
                    (current_num, current) = fs
                        .resolve_dir_entry(found_entry)
                        .map_err(|_| VfsError::NoSuchFile)?;
                }
            }
        }

        Ok((current_num, current))
    }
}

impl<T> FileSystem for VirtualExt2Fs<T>
where
    T: BlockDevice + Send + Sync,
{
    fn fsid(&self) -> FsId {
        self.fsid
    }

    fn open(&mut self, path: &Path) -> Result<VfsHandle> {
        // FIXME: instead of returning a new handle, check whether we already have that inode open behind another handle
        let (found_num, found) = self.find_inode(path)?;
        let handle = next_handle();
        let inode = Ext2Inode::new(self.fsid(), self.inner.clone(), found_num, found);
        self.handles.insert(handle, Arc::new(RwLock::new(inode)));
        Ok(handle)
    }

    fn close(&mut self, handle: VfsHandle) -> Result<()> {
        self.handles.remove(&handle).ok_or(VfsError::HandleClosed)?;
        Ok(())
    }

    fn read_dir(&mut self, path: &Path) -> Result<Vec<DirEntry>> {
        let node = self.find_inode(path)?.1;
        Ok(self
            .inner
            .read()
            .list_dir(&node)
            .map_err(|_| VfsError::NoSuchFile)?
            .into_iter()
            .filter(|entry| entry.name().is_some()) // TODO: could be none if the name is not valid utf8, we should maybe handle that differently
            .filter(|entry| entry.typ().is_some()) // TODO: dir entries not necessarily have a type, do we want to support that?
            .map(|entry| {
                let name = entry.name().unwrap().to_string();
                let ext2_type = entry.typ().unwrap();
                DirEntry::new(name, ext2_type.into())
            })
            .collect())
    }

    fn read(&mut self, handle: VfsHandle, buf: &mut [u8], offset: usize) -> Result<usize> {
        self.resolve_handle(handle)?.read().read(buf, offset)
    }

    fn write(&mut self, handle: VfsHandle, buf: &[u8], offset: usize) -> Result<usize> {
        self.resolve_handle(handle)?.write().write(buf, offset)
    }

    fn truncate(&mut self, _handle: VfsHandle, _size: usize) -> Result<()> {
        todo!()
    }

    fn stat(&mut self, handle: VfsHandle, stat: &mut Stat) -> Result<()> {
        self.resolve_handle(handle)?.read().stat(stat)
    }

    fn create(&mut self, _path: &Path, _ftype: FileType) -> Result<()> {
        todo!()
    }

    fn remove(&mut self, _path: &Path) -> Result<()> {
        todo!()
    }
}

impl From<ext2::DirType> for FileType {
    fn from(value: ext2::DirType) -> Self {
        match value {
            x if x == ext2::DirType::Directory => FileType::Directory,
            x if x == ext2::DirType::RegularFile => FileType::RegularFile,
            x if x == ext2::DirType::CharacterDevice => FileType::CharacterDevice,
            x if x == ext2::DirType::BlockDevice => FileType::BlockDevice,
            x if x == ext2::DirType::FIFO => FileType::FIFO,
            x if x == ext2::DirType::UnixSocket => FileType::Socket,
            x if x == ext2::DirType::SymLink => FileType::SymbolicLink,
            _ => unreachable!(),
        }
    }
}
