use core::slice::{from_raw_parts, from_raw_parts_mut};

use kernel_api::syscall::{Errno, Syscall};

use crate::process::fd::Fileno;
use crate::syscall::convert::{TryFromUserspaceAddress, UserspaceAddress};
use crate::syscall::error::Result;
use crate::syscall::{
    sys_access, sys_close, sys_exit, sys_mmap, sys_read, sys_write, MapFlags, Prot,
};
use crate::syscall::{sys_open, AMode};

/// Dispatches syscalls. Inputs are the raw register values, the return value
/// is the result of the syscall that is identified by the [`syscall`] argument.
// not unsafe because the caller can't do much about the argument validity anyways
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
        Syscall::Open => dispatch_sys_open(arg1, arg2, arg3)
            .map(|f| f.as_usize())
            .map(Errno::from),
        Syscall::Read => dispatch_sys_read(arg1, arg2, arg3).map(Errno::from),
        Syscall::Write => dispatch_sys_write(arg1, arg2, arg3).map(Errno::from),
        _ => Err(Errno::ENOSYS), // not implemented
    };
    syscall_result.unwrap_or_else(|v| v).as_isize()
}

fn dispatch_sys_access(arg1: usize, arg2: usize) -> Result<()> {
    let userspace_addr = UserspaceAddress::try_from(arg1).map_err(|_| Errno::EINVAL)?;
    let path = <&str as TryFromUserspaceAddress>::try_from_userspace_addr(userspace_addr)?;

    sys_access(path, AMode::from_bits_truncate(arg2))
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
    let buf = unsafe { from_raw_parts_mut(arg2 as *mut u8, arg3) };
    sys_read(Fileno::new(arg1), buf)
}

fn dispatch_sys_write(arg1: usize, arg2: usize, arg3: usize) -> Result<usize> {
    let buf = unsafe { from_raw_parts(arg2 as *const u8, arg3) };
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
