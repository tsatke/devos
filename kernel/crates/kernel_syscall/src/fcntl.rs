use crate::UserspacePtr;
use alloc::borrow::ToOwned;
use core::ffi::CStr;
use core::slice::from_raw_parts;
use kernel_abi::{Errno, EACCES, EINVAL, ENAMETOOLONG, ENOSYS};
use kernel_vfs::path::{AbsoluteOwnedPath, AbsolutePath, Path};
use kernel_vfs::{OpenError, Vfs};
use spin::rwlock::RwLock;

pub trait VfsAccess {
    fn vfs(&self) -> &RwLock<Vfs>;
}

pub trait Process {
    fn current_dir(&self) -> &RwLock<AbsoluteOwnedPath>;
}

pub trait ProcessAccess {
    type Process: Process;

    fn current_process(&self) -> &Self::Process;
}

pub trait Task {}

pub trait TaskAccess {
    type Task: Task;

    fn current_task(&self) -> &Self::Task;
}

/// Open a file at the given path with the specified flags and mode.
/// This is the kernel side implementation of [`open`] in [`POSIX.1-2024`].
///
/// [`open`]: https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html
/// [`POSIX.1-2024`]: https://pubs.opengroup.org/onlinepubs/9799919799
pub fn sys_open<Cx: ProcessAccess + VfsAccess>(
    cx: &Cx,
    path: UserspacePtr<u8>,
    _oflag: i32,
    _mode: i32,
) -> Result<usize, Errno> {
    let path = {
        let path_bytes_max = unsafe { from_raw_parts(*path, 4096) };
        let path = CStr::from_bytes_until_nul(path_bytes_max).map_err(|_| ENAMETOOLONG)?;
        let path = path.to_str().map_err(|_| EINVAL)?;
        let path = Path::new(path);
        match AbsolutePath::try_new(path) {
            Ok(p) => p.to_owned(),
            Err(_) => {
                let mut p = cx.current_process().current_dir().read().clone();
                p.push(path);
                p
            }
        }
    };

    // TODO: check permissions, flags like O_CREAT etc.

    let vfs = cx.vfs();
    let node = vfs.read().open(&path).map_err(|e| match e {
        OpenError::NotFound => EACCES,
    })?;

    // TODO: implement
    let _ = node;
    Err(ENOSYS)
}
