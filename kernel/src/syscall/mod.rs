use alloc::format;
use alloc::vec::Vec;
use core::ops::BitAnd;

use bitflags::bitflags;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

pub use dispatch::*;
pub use error::*;
use kernel_api::syscall::{Errno, FfiSockAddr, FileMode, SocketDomain, SocketType, Stat};

use crate::{process, serial_println};
use crate::io::path::Path;
use crate::io::socket::create_socket;
use crate::io::vfs::vfs;
use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::process::fd::Fileno;
use crate::process::vmm;

mod convert;
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

pub enum FsType {
    Ext2,
}

pub fn sys_access(path: impl AsRef<Path>, amode: AMode) -> Result<()> {
    if amode != AMode::F_OK {
        // TODO: support permissions
        return Err(Errno::ENOSYS);
    }

    vfs().stat_path(path, &mut Stat::default()).map_err(Into::into).map(|_| ())
}

pub fn sys_close(fd: Fileno) -> Result<()> {
    serial_println!("sys_close({})", fd);
    let process = process::current();
    process.close_fd(fd).map_err(Into::into)
}

pub fn sys_dup(fd: Fileno) -> Result<Fileno> {
    serial_println!("sys_dup({:?})", fd);

    let process = process::current();
    let node = process
        .open_fds()
        .read()
        .get(&fd)
        .map(|desc| desc.node().clone())
        .ok_or(Errno::EBADF)?;
    let new_fd = process.get_fileno_for(node);
    Ok(new_fd)
}

pub fn sys_execve(path: impl AsRef<Path>, argv: &[&str], envp: &[&str]) -> Result<!> {
    serial_println!("sys_execve({:?}, {:?}, {:?})", path.as_ref(), argv, envp);

    unimplemented!("sys_execve")
}

pub fn sys_exit(status: usize) -> ! {
    serial_println!("sys_exit({})", status);
    process::exit_thread();
}

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Prot : u32 {
        const None = 0x0;
        const Read = 0x1;
        const Write = 0x2;
        const Exec = 0x4;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct MapFlags : u32 {
        const Shared = 0x1;
        const Private = 0x2;
        const Fixed = 0x4;
        const Anon = 0x8;
    }
}

pub fn sys_mmap(
    addr: VirtAddr,
    size: usize,
    prot: Prot,
    map_flags: MapFlags,
    fd: Fileno,
    offset: usize,
) -> Result<VirtAddr> {
    serial_println!(
        "sys_mmap({:#x}, {}, {:?}, {:?}, {:?}, {})",
        addr,
        size,
        prot,
        map_flags,
        fd,
        offset
    );

    let addr = if addr.is_null() { MapAt::Anywhere } else { MapAt::Fixed(addr) };

    let mut flags = PageTableFlags::empty();
    if prot.contains(Prot::Read) {
        flags |= PageTableFlags::PRESENT;
    }
    if prot.contains(Prot::Write) {
        flags |= PageTableFlags::WRITABLE;
    }
    if !prot.contains(Prot::Exec) {
        flags |= PageTableFlags::NO_EXECUTE;
    }

    let mapped_address = if map_flags.contains(MapFlags::Anon) {
        vmm()
            .allocate_memory_backed_vmobject(
                format!("mmap anon (len={})", size),
                addr,
                size,
                AllocationStrategy::AllocateOnAccess,
                flags,
            )
            .map_err(|_| Errno::ENOMEM)?
    } else {
        let process = process::current();
        let node = process
            .open_fds()
            .read()
            .get(&fd)
            .ok_or(Errno::EBADF)?
            .node()
            .clone();

        let mut stat = Stat::default();
        vfs().stat_path(node.path(), &mut stat).map_err(Into::<Errno>::into)?;

        if stat.mode.is_regular_file() {
            vmm()
                .allocate_file_backed_vm_object(
                    format!("mmap '{}' (offset={}, len={})", node.path(), offset, size),
                    node,
                    offset,
                    addr,
                    size,
                    flags,
                )
                .map_err(|_| Errno::ENOMEM)?
        } else {
            // check whether the file is a device and needs special handling
            let fs = node.fs().read();
            let res = if let Some(phys_frames) = fs.physical_memory(node.handle())? {
                let frames = phys_frames.collect::<Vec<_>>();
                vmm()
                    .allocate_memory_backed_vmobject(
                        format!("mmap device '{}' (len={})", node.path(), size),
                        addr,
                        size,
                        AllocationStrategy::MapNow(&frames),
                        flags,
                    )
                    .map_err(|_| Errno::ENOMEM)?
            } else {
                // we have some non-regular file that doesn't have physical memory, what?
                panic!("mmap unsupported file type: {:#?} (doesn't have physical memory)", stat.mode.bitand(FileMode::S_IFMT));
            };
            res
        }
    };

    Ok(mapped_address)
}

pub fn sys_mount(
    _source: impl AsRef<Path>,
    _target: impl AsRef<Path>,
    _fstype: FsType,
    _mountflags: MountFlags,
) -> Result<()> {
    Err(Errno::ENOSYS)
}

// TODO: OpenFlags and Mode
pub fn sys_open(path: impl AsRef<Path>, flags: usize, mode: usize) -> Result<Fileno> {
    serial_println!(
        "sys_open({:#p} ({}), {}, {})",
        path.as_ref().as_ptr(),
        path.as_ref(),
        flags,
        mode
    );
    let process = process::current();
    process.open_file(&path).map_err(Into::into)
}

pub fn sys_read(fd: Fileno, buf: &mut [u8]) -> Result<usize> {
    serial_println!("sys_read({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    let process = process::current();
    process.read(fd, buf).map_err(Into::into)
}

pub fn sys_write(fd: Fileno, buf: &[u8]) -> Result<usize> {
    serial_println!("sys_write({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    let process = process::current();
    process.write(fd, buf).map_err(Into::into)
}

pub fn sys_socket(domain: SocketDomain, typ: SocketType, protocol: usize) -> Result<usize> {
    serial_println!("sys_socket({:?}, {:?}, {})", domain, typ, protocol);
    let socket_id = create_socket();

    Ok(socket_id.into_usize())
}

pub fn sys_bind(socket: usize, address: FfiSockAddr, address_len: usize) -> Result<()> {
    serial_println!("sys_bind({}, {:?}, {})", socket, address, address_len);

    Ok(())
}

pub fn sys_stat(path: impl AsRef<Path>, stat: &mut Stat) -> Result<()> {
    serial_println!("sys_stat({:?}, {:#p})", path.as_ref(), stat);

    vfs().stat_path(path, stat).map_err(Into::into).map(|_| ())
}