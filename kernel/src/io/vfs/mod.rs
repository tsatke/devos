use alloc::borrow::ToOwned;
use alloc::string::ToString;
use core::mem::MaybeUninit;

mod error;
mod inode;
mod memfs;
mod perm;
mod root;

pub mod ext2;

pub use error::*;
pub use inode::*;
pub use perm::*;
use root::RootDir;

use crate::io::path::{Component, Path};
use crate::io::vfs::memfs::MemFs;

static mut VFS: MaybeUninit<Vfs> = MaybeUninit::uninit();

pub fn init() {
    unsafe {
        VFS.write(Vfs::new());
    }

    mount("/", MemFs::new("mem".to_string()).root_inode()).unwrap();
}

pub fn mount(p: impl AsRef<Path>, node: Inode) -> Result<(), MountError> {
    unsafe { vfs() }.mount(p, node)
}

pub fn find(p: impl AsRef<Path>) -> Result<Inode, LookupError> {
    unsafe { vfs() }.find(p)
}

unsafe fn vfs() -> &'static Vfs {
    unsafe { VFS.assume_init_ref() }
}

pub struct Vfs {
    root: Inode,
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
        }
    }

    fn mount(&self, p: impl AsRef<Path>, node: Inode) -> Result<(), MountError> {
        let target_node = match self.find(p) {
            Ok(n) => n,
            Err(e) => return Err(e)?,
        };
        let dir = match target_node.clone() {
            Inode::File(_) => return Err(MountError::NotDirectory),
            Inode::Dir(d) => d,
            Inode::BlockDevice(_) => return Err(MountError::NotDirectory),
            Inode::CharacterDevice(_) => return Err(MountError::NotDirectory),
            Inode::Symlink(link) => {
                let guard = link.read();
                let target_path = guard.target_path()?;
                let symlink_target_node =
                    Self::find_inode_from(target_path.as_path(), target_node)?;
                if !matches!(symlink_target_node, Inode::Dir(_)) {
                    return Err(MountError::NotDirectory);
                }
                symlink_target_node.as_dir().unwrap()
            }
        };
        let mut guard = dir.write();
        guard.mount(node)
    }

    fn find(&self, p: impl AsRef<Path>) -> Result<Inode, LookupError> {
        let path = p.as_ref().to_owned();
        let mut components = path.components();

        // check that we got an absolute path
        let first = components.next();
        if !first.is_some_and(|e| matches!(e, Component::RootDir)) {
            return Err(LookupError::NoSuchEntry);
        }

        Self::find_inode_from(p, self.root.clone())
    }

    fn find_inode_from(p: impl AsRef<Path>, starting_point: Inode) -> Result<Inode, LookupError> {
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
                        Inode::File(_) => return Err(LookupError::NoSuchEntry),
                        Inode::Dir(d) => d,
                        Inode::BlockDevice(_) => return Err(LookupError::NoSuchEntry),
                        Inode::CharacterDevice(_) => return Err(LookupError::NoSuchEntry),
                        Inode::Symlink(link) => {
                            let guard = link.read();
                            let target_path = guard.target_path()?;
                            let target_node =
                                Self::find_inode_from(target_path.as_path(), current)?;
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
                }
            };
        }

        // we found the vnode
        Ok(current)
    }
}
