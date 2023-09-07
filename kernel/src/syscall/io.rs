use crate::io::path::Path;
use crate::io::vfs::find;
use crate::serial_println;
use bitflags::bitflags;
use kernel_api::syscall::{Errno, ENOENT, ENOSYS, OK};

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct AMode: usize {
        const F_OK = 0;
        const X_OK = 1;
        const W_OK = 2;
        const R_OK = 4;
    }
}

pub fn sys_access(path: impl AsRef<Path>, amode: AMode) -> Errno {
    if amode != AMode::F_OK {
        // TODO: support permissions
        return ENOSYS;
    }

    if find(path).is_ok() {
        OK
    } else {
        ENOENT
    }
}

pub enum FsType {
    Ext2,
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

pub fn sys_mount(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    fstype: FsType,
    mountflags: MountFlags,
) -> Errno {
    ENOSYS
}

pub fn sys_read(fd: usize, buf: &mut [u8]) -> Errno {
    serial_println!("sys_read({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    buf[0] = 1;
    1.into()
}

pub fn sys_write(fd: usize, buf: &[u8]) -> Errno {
    serial_println!("sys_write({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    ENOSYS
}

// TODO: OpenFlags and Mode
pub fn sys_open(path: impl AsRef<Path>, flags: usize, mode: usize) -> Errno {
    serial_println!(
        "sys_open({:#p} ({}), {}, {})",
        path.as_ref().as_ptr(),
        path.as_ref(),
        flags,
        mode
    );
    ENOSYS
}

pub fn sys_close(fd: usize) -> Errno {
    serial_println!("sys_close({})", fd);
    ENOSYS
}
