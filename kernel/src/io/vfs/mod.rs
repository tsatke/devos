use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use root::RootDir;
use spin::RwLock;

mod devfs;
mod error;
mod inode;
mod memfs;
mod perm;
mod root;

pub mod ext2;

pub use error::*;
pub use inode::*;
pub use perm::*;

use crate::io::path::{Component, Path};
use crate::io::vfs::devfs::DevFs;
use crate::io::vfs::ext2::Ext2Fs;
use crate::syscall::unistd::{sys_access, AMode};

static mut VFS: Option<RwLock<Vfs>> = None;

static FSID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    if unsafe { VFS.is_some() } {
        panic!("vfs already initialized");
    }

    let root_drive = ide::drives()
        .nth(1) // TODO: for now, [0] is the boot drive, [1] is the os disk
        .expect("we need at least one additional IDE drive for now")
        .clone();

    let rootfs_dev = ::ext2::Ext2Fs::try_new(root_drive).expect("root drive must be ext2 for now");
    let rootfs = Ext2Fs::new(FSID_COUNTER.fetch_add(1, Relaxed), rootfs_dev);

    unsafe {
        let mut vfs = Vfs::new();
        vfs.set_root(rootfs.root_inode().expect("unable to read root inode"));
        VFS = Some(RwLock::new(vfs));
    }

    if !sys_access("/dev", AMode::F_OK).is_ok() {
        todo!("create /dev");
    }

    mount("/dev", DevFs::new(FSID_COUNTER.fetch_add(1, Relaxed))).expect("unable to mount devfs");
}

pub fn mount(p: impl AsRef<Path>, fs: impl Fs) -> Result<(), MountError> {
    vfs().write().mount(p, fs)
}

pub fn find(p: impl AsRef<Path>) -> Result<Inode, LookupError> {
    vfs().read().find(p)
}

pub fn find_from(p: impl AsRef<Path>, starting_point: Inode) -> Result<Inode, LookupError> {
    vfs().read().find_inode_from(p, starting_point)
}

fn vfs() -> &'static RwLock<Vfs> {
    unsafe {
        // SAFETY: this only creates an immutable reference
        VFS.as_ref().expect("vfs not initialized")
    }
}

pub struct Vfs {
    root: Inode,
    mounts: BTreeMap<(u64, InodeNum), Inode>,
}

impl Vfs {
    fn new() -> Self {
        Vfs {
            root: Inode::new_dir(RootDir::new(
                "/".into(),
                Stat {
                    inode: 0_u64.into(),
                    ..Default::default()
                },
            )),
            mounts: BTreeMap::new(),
        }
    }

    fn set_root(&mut self, root: Inode) {
        self.root = root;
    }

    fn mount(&mut self, p: impl AsRef<Path>, fs: impl Fs) -> Result<(), MountError> {
        let mountee = fs.root_inode();
        let mount_point = self.find(p).map_err(MountError::LookupError)?;

        self.mounts
            .insert((mount_point.stat().dev, mount_point.num()), mountee);
        Ok(())
    }

    fn find(&self, p: impl AsRef<Path>) -> Result<Inode, LookupError> {
        let path = p.as_ref().to_owned();
        let mut components = path.components();

        // check that we got an absolute path
        let first = components.next();
        if !first.is_some_and(|e| matches!(e, Component::RootDir)) {
            return Err(LookupError::NoSuchEntry);
        }

        self.find_inode_from(p, self.root.clone())
    }

    fn find_inode_from(
        &self,
        p: impl AsRef<Path>,
        starting_point: Inode,
    ) -> Result<Inode, LookupError> {
        let path = p.as_ref().to_owned();
        let components = path.components();

        let mut current = starting_point;
        let mut seen_root = false;
        for component in components {
            match component {
                Component::RootDir => {
                    if seen_root {
                        panic!("unexpected root dir in the middle of a path");
                    }
                    seen_root = true;
                }
                Component::CurrentDir => {} // do nothing
                Component::ParentDir => {
                    todo!("parent dir");
                }
                Component::Normal(v) => {
                    let current_dir = match current.clone() {
                        Inode::File(_) | Inode::BlockDevice(_) | Inode::CharacterDevice(_) => {
                            return Err(LookupError::NoSuchEntry)
                        }
                        Inode::Dir(d) => d,
                        Inode::Symlink(link) => {
                            let guard = link.read();
                            let target_path = guard.target_path()?;
                            let target_node =
                                self.find_inode_from(target_path.as_path(), current)?;
                            if !matches!(target_node, Inode::Dir(_)) {
                                return Err(LookupError::NoSuchEntry);
                            }
                            current = target_node;
                            continue; // try again with the resolved symlink as current node
                        }
                    };
                    let next_element = current_dir.read().lookup(&v);
                    let new_current = match next_element {
                        Ok(node) => node,
                        Err(_) => return Err(LookupError::NoSuchEntry),
                    };
                    current = new_current;

                    // check if current is a mount point
                    let id = (current.stat().dev, current.num());
                    if let Some(inode) = self.mounts.get(&id) {
                        current = inode.clone();
                    }
                }
            };
        }

        // we found the vnode
        Ok(current)
    }
}
