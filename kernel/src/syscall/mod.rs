use alloc::string::ToString;
use alloc::{format, vec};
use core::intrinsics::transmute;

use bitflags::bitflags;
use elfloader::ElfBinary;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

pub use dispatch::*;
pub use error::*;
use kernel_api::syscall::Errno;

use crate::io::path::Path;
use crate::io::vfs::{vfs, Stat};
use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::process::elf::ElfLoader;
use crate::process::fd::Fileno;
use crate::process::vmm;
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

pub enum FsType {
    Ext2,
}

pub fn sys_access(path: impl AsRef<Path>, amode: AMode) -> Result<Stat> {
    if amode != AMode::F_OK {
        // TODO: support permissions
        return Err(Errno::ENOSYS);
    }

    vfs().stat_path(path).map_err(Into::into)
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

    let strategy = AllocationStrategy::AllocateOnAccess;
    if map_flags.contains(MapFlags::Anon) {
        vmm()
            .allocate_memory_backed_vmobject(
                format!("mmap anon (len={})", size),
                MapAt::Fixed(addr),
                size,
                strategy,
                flags,
            )
            .map_err(|_| Errno::ENOMEM)?;
    } else {
        let process = process::current();
        let node = process
            .open_fds()
            .read()
            .get(&fd)
            .ok_or(Errno::EBADF)?
            .node()
            .clone();
        vmm()
            .allocate_file_backed_vm_object(
                format!("mmap '{}' (offset={}, len={})", node.path(), offset, size),
                node,
                offset,
                MapAt::Fixed(addr),
                size,
                flags,
            )
            .map_err(|_| Errno::ENOMEM)?;
    };

    Ok(addr)
}

pub fn sys_munmap(addr: VirtAddr) -> Result<()> {
    serial_println!("sys_munmap({:#x})", addr);
    let mut guard = vmm().vm_objects().write();
    guard.remove(&addr);
    Ok(())
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
