use core::ffi::CStr;
use core::ptr;
use core::slice::from_raw_parts;

use kernel_api::PATH_MAX;
use kernel_api::syscall::{Errno, FfiSockAddr, SocketDomain, SocketType, Stat, Syscall};

use crate::process::fd::Fileno;
use crate::syscall::{MapFlags, Prot, sys_access, sys_bind, sys_close, sys_exit, sys_mmap, sys_read, sys_socket, sys_stat, sys_write};
use crate::syscall::{AMode, sys_open};
use crate::syscall::convert::{
    TryFromUserspaceAddress, TryFromUserspaceRange, UserspaceAddress, UserspaceRange,
};
use crate::syscall::error::Result;

fn check_is_userspace(arg: usize) -> Result<()> {
    UserspaceAddress::try_from(arg).map_err(|_| Errno::EINVAL)?;
    Ok(())
}

/// Dispatches syscalls. Inputs are the raw register values, the return value
/// is the result of the syscall that is identified by the [`syscall`] argument.
// not unsafe because the caller can't do much about the argument validity anyway
pub fn dispatch_syscall(
    syscall: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let syscall = match TryInto::<Syscall>::try_into(syscall) {
        Ok(v) => v,
        Err(_) => return Errno::ENOSYS.as_isize(),
    };

    let syscall_result = match syscall {
        Syscall::Access => dispatch_sys_access(arg1, arg2).map(Errno::from),
        Syscall::Close => dispatch_sys_close(arg1).map(Errno::from),
        Syscall::Exit => dispatch_sys_exit(arg1),
        Syscall::Mmap => dispatch_sys_mmap(arg1, arg2, arg3, arg4, arg5, arg6).map(Errno::from),
        Syscall::Open => dispatch_sys_open(arg1, arg2, arg3).map(Errno::from),
        Syscall::Read => dispatch_sys_read(arg1, arg2, arg3).map(Errno::from),
        Syscall::Write => dispatch_sys_write(arg1, arg2, arg3).map(Errno::from),
        Syscall::Socket => dispatch_sys_socket(arg1, arg2, arg3).map(Errno::from),
        Syscall::Bind => dispatch_sys_bind(arg1, arg2, arg3).map(Errno::from),
        Syscall::Stat => dispatch_sys_stat(arg1, arg2).map(Errno::from),
    };
    syscall_result.unwrap_or_else(|v| v).as_isize()
}

fn dispatch_sys_access(arg1: usize, arg2: usize) -> Result<()> {
    let userspace_addr = UserspaceAddress::try_from(arg1).map_err(|_| Errno::EINVAL)?;
    let path = <&str as TryFromUserspaceAddress>::try_from_userspace_addr(userspace_addr)?;

    sys_access(path, AMode::from_bits_truncate(arg2))
}

fn dispatch_sys_stat(arg1: usize, arg2: usize) -> Result<()> {
    check_is_userspace(arg1)?;

    let path = CStr::from_bytes_until_nul(unsafe { from_raw_parts(arg1 as *const u8, PATH_MAX) })
        .map_err(|_| Errno::EINVAL)?
        .to_str()
        .map_err(|_| Errno::EINVAL)?;
    let mut stat = unsafe { &mut *(arg2 as *mut Stat) };

    sys_stat(path, &mut stat)
}

fn dispatch_sys_mmap(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> Result<usize> {
    let addr = UserspaceAddress::try_from(arg1).map_err(|_| Errno::EINVAL)?;
    let len = arg2;
    let prot = Prot::from_bits_truncate(arg3 as u32);
    let flags = MapFlags::from_bits_truncate(arg4 as u32);
    let fd = Fileno::new(arg5);
    let offset = arg6;

    sys_mmap(*addr, len, prot, flags, fd, offset).map(|addr| addr.as_u64() as usize)
}

fn dispatch_sys_read(arg1: usize, arg2: usize, arg3: usize) -> Result<usize> {
    let ptr = UserspaceAddress::try_from(arg2).map_err(|_| Errno::EINVAL)?;
    let range = UserspaceRange::try_from(ptr, arg3).map_err(|_| Errno::EINVAL)?;
    let buf = <&mut [u8] as TryFromUserspaceRange>::try_from_userspace_range(range)?;

    sys_read(Fileno::new(arg1), buf)
}

fn dispatch_sys_write(arg1: usize, arg2: usize, arg3: usize) -> Result<usize> {
    let ptr = UserspaceAddress::try_from(arg2).map_err(|_| Errno::EINVAL)?;
    let range = UserspaceRange::try_from(ptr, arg3).map_err(|_| Errno::EINVAL)?;
    let buf = <&[u8] as TryFromUserspaceRange>::try_from_userspace_range(range)?;

    sys_write(Fileno::new(arg1), buf)
}

fn dispatch_sys_open(arg1: usize, arg2: usize, arg3: usize) -> Result<Fileno> {
    let userspace_addr = UserspaceAddress::try_from(arg1).map_err(|_| Errno::EINVAL)?;
    let path = <&str as TryFromUserspaceAddress>::try_from_userspace_addr(userspace_addr)?;

    sys_open(path, arg2, arg3)
}

fn dispatch_sys_close(arg1: usize) -> Result<()> {
    sys_close(Fileno::new(arg1))
}

fn dispatch_sys_exit(arg1: usize) -> ! {
    sys_exit(arg1)
}

fn dispatch_sys_socket(arg1: usize, arg2: usize, arg3: usize) -> Result<usize> {
    let domain = TryInto::<SocketDomain>::try_into(arg1).map_err(|_| Errno::EINVAL)?;
    let typ = TryInto::<SocketType>::try_into(arg2).map_err(|_| Errno::EINVAL)?;
    let protocol = arg3;

    sys_socket(domain, typ, protocol)
}

fn dispatch_sys_bind(arg1: usize, arg2: usize, arg3: usize) -> Result<()> {
    let socket = arg1;
    let address = unsafe { ptr::read(arg2 as *const FfiSockAddr) };
    let address_len = arg3;

    sys_bind(socket, address, address_len)
}