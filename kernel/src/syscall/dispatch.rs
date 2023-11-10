use core::slice::{from_raw_parts, from_raw_parts_mut};

use x86_64::VirtAddr;

use kernel_api::syscall::{Errno, Syscall};

use crate::process::fd::Fileno;
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

    unsafe {
        match syscall {
            Syscall::Access => dispatch_sys_access(arg1, arg2),
            Syscall::Close => dispatch_sys_close(arg1),
            Syscall::Exit => dispatch_sys_exit(arg1),
            Syscall::Mmap => dispatch_sys_mmap(arg1, arg2, arg3, arg4, arg5, arg6),
            Syscall::Open => dispatch_sys_open(arg1, arg2, arg3),
            Syscall::Read => dispatch_sys_read(arg1, arg2, arg3),
            Syscall::Write => dispatch_sys_write(arg1, arg2, arg3),
            _ => Errno::ENOSYS, // not implemented
        }
    }
    .as_isize()
}

unsafe fn dispatch_sys_access(arg1: usize, arg2: usize) -> Errno {
    let ptr = arg1 as *const u8; // FIXME: check that `path` points to userspace data

    // FIXME: use PATH_MAX instead of hard coded 255
    let len = match strlen_s(ptr, 255) {
        None => return Errno::ENAMETOOLONG,
        Some(v) => v,
    };
    let path = match core::str::from_utf8(unsafe { from_raw_parts(ptr, len) }) {
        Ok(v) => v,
        Err(_) => return Errno::EINVAL,
    };
    match sys_access(path, AMode::from_bits_truncate(arg2)) {
        Ok(_v) => todo!("convert Stat into Errno"), // TODO: probably copy to process address space and create pointer
        Err(e) => e,
    }
}

fn dispatch_sys_mmap(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> Errno {
    let addr = match VirtAddr::try_new(arg1 as u64) {
        Ok(v) => v,
        Err(_) => return Errno::EINVAL,
    };
    let len = arg2;
    let prot = Prot::from_bits_truncate(arg3 as u32);
    let flags = MapFlags::from_bits_truncate(arg4 as u32);
    let fd = Fileno::new(arg5);
    let offset = arg6;

    sys_mmap(addr, len, prot, flags, fd, offset)
        .map(|addr| addr.as_u64() as usize)
        .into()
}

unsafe fn dispatch_sys_read(arg1: usize, arg2: usize, arg3: usize) -> Errno {
    let buf = unsafe { from_raw_parts_mut(arg2 as *mut u8, arg3) };
    sys_read(arg1, buf).into()
}

unsafe fn dispatch_sys_write(arg1: usize, arg2: usize, arg3: usize) -> Errno {
    let buf = unsafe { from_raw_parts(arg2 as *const u8, arg3) };
    sys_write(arg1, buf).into()
}

unsafe fn dispatch_sys_open(arg1: usize, arg2: usize, arg3: usize) -> Errno {
    let path = unsafe { *(arg1 as *const &str) };
    sys_open(path, arg2, arg3)
        .map(|fileno| fileno.as_usize())
        .into()
}

fn dispatch_sys_close(arg1: usize) -> Errno {
    sys_close(Fileno::new(arg1)).into()
}

fn dispatch_sys_exit(arg1: usize) -> ! {
    sys_exit(arg1)
}

fn strlen_s(ptr: *const u8, max: usize) -> Option<usize> {
    (0..max).find(|&i| unsafe { *ptr.add(i) } == 0)
}
