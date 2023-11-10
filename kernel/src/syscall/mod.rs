use alloc::string::ToString;
use alloc::vec;
use core::intrinsics::transmute;

use bitflags::bitflags;
use elfloader::ElfBinary;

pub use dispatch::*;
pub use error::*;
use kernel_api::syscall::Errno;

use crate::io::path::Path;
use crate::io::vfs::{vfs, Stat, VfsError};
use crate::process::elf::ElfLoader;
use crate::{process, serial_println};

mod dispatch;
mod error;

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct AMode: usize {
        const F_OK = 0;
        const X_OK = 1;
        const W_OK = 2;
        const R_OK = 4;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct MountFlags: u32 {
        /// Mount read-only
        const MS_RDONLY = 0x01;
        /// Ignore suid and sgid bits
        const MS_NOSUID = 0x02;
        /// Disallow access to device special files
        const MS_NODEV = 0x04;
        /// Disallow program execution
        const MS_NOEXEC = 0x08;
        /// Writes are synced at once
        const MS_SYNCHRONOUS = 0x10;
        /// Alter flags of a mounted FS
        const MS_REMOUNT = 0x20;
        /// Allow mandatory locks on an FS
        const MS_MANDLOCK = 0x40;
        /// Directory modifications are synchronous
        const MS_DIRSYNC = 0x80;
        /// Do not update access times
        const MS_NOATIME = 0x400;
        /// Do not update directory access times
        const MS_NODIRATIME = 0x800;
        /// Bind directory at different place
        const MS_BIND = 0x1000;
        /// Move subtree
        const MS_MOVE = 0x2000;
        /// Recursive loop
        const MS_REC = 0x4000;
        /// Silent filesystem errors
        const MS_SILENT = 0x8000;
        /// VFS does not apply the umask
        const MS_POSIXACL = 0x10000;
        /// Change to unbindable
        const MS_UNBINDABLE = 0x20000;
        /// Change to private
        const MS_PRIVATE = 0x40000;
        /// Change to slave
        const MS_SLAVE = 0x80000;
        /// Change to shared
        const MS_SHARED = 0x100000;
        /// Update atime relative to mtime/ctime
        const MS_RELATIME = 0x200000;
        /// This is a kern_mount call
        const MS_KERNMOUNT = 0x400000;
        /// Update inode I_version field
        const MS_I_VERSION = 0x800000;
        /// Always perform atime updates
        const MS_STRICTATIME = 0x1000000;
        /// Update timestamps on write access
        const MS_LAZYTIME = 0x2000000;
    }
}

// TODO: OpenFlags and Mode
pub fn sys_open(path: impl AsRef<Path>, flags: usize, mode: usize) -> Result<usize> {
    serial_println!(
        "sys_open({:#p} ({}), {}, {})",
        path.as_ref().as_ptr(),
        path.as_ref(),
        flags,
        mode
    );
    Ok(0)
}

pub enum FsType {
    Ext2,
}

pub fn sys_access(path: impl AsRef<Path>, amode: AMode) -> Result<Stat> {
    if amode != AMode::F_OK {
        // TODO: support permissions
        return Err(Errno::ENOSYS);
    }

    match vfs().stat_path(path) {
        Ok(v) => Ok(v),
        Err(e) => Err(match e {
            VfsError::NoSuchFileSystem => Errno::ENXIO,
            VfsError::NoSuchFile => Errno::ENOENT,
            VfsError::Unsupported => Errno::ENOSYS,
            VfsError::HandleClosed => Errno::EBADF,
            VfsError::ReadError => Errno::EIO,
        }),
    }
}

pub fn sys_close(fd: usize) -> Result<()> {
    serial_println!("sys_close({})", fd);
    Err(Errno::ENOSYS)
}

pub fn sys_execve(path: impl AsRef<Path>, argv: &[&str], envp: &[&str]) -> Result<!> {
    serial_println!("sys_execve({:?}, {:?}, {:?})", path.as_ref(), argv, envp);

    let path = path.as_ref();

    let elf_data = {
        let file = vfs().open(path).map_err(|_| Errno::ENOENT)?;
        let stat = vfs().stat(&file).map_err(|_| Errno::EIO)?;
        let size = stat.size;
        let mut buf = vec![0_u8; size as usize];
        vfs().read(&file, &mut buf, 0).map_err(|_| Errno::EIO)?;
        buf
    };

    let mut loader = ElfLoader::default();
    let elf = ElfBinary::new(&elf_data).unwrap();
    elf.load(&mut loader).unwrap();
    let image = loader.into_inner();
    let entry = unsafe { image.as_ptr().add(elf.entry_point() as usize) };
    let entry_fn = unsafe { transmute(entry) };

    // execute the executable in the new task...
    process::spawn_task_in_current_process(path.to_string(), entry_fn);
    // ...and stop the current task
    unsafe { process::exit_current_task() }
}

pub fn sys_exit(status: usize) -> ! {
    serial_println!("sys_exit({})", status);
    process::exit();
}

pub fn sys_mount(
    _source: impl AsRef<Path>,
    _target: impl AsRef<Path>,
    _fstype: FsType,
    _mountflags: MountFlags,
) -> Result<()> {
    Err(Errno::ENOSYS)
}

pub fn sys_read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    serial_println!("sys_read({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    buf[0] = 1;
    Ok(1)
}

pub fn sys_write(fd: usize, buf: &[u8]) -> Result<usize> {
    serial_println!("sys_write({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    Err(Errno::ENOSYS)
}
