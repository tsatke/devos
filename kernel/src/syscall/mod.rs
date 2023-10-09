use core::slice::{from_raw_parts, from_raw_parts_mut};
use kernel_api::syscall::{Errno, Syscall, EINVAL, ENAMETOOLONG, ENOSYS};

use crate::syscall::io::*;

pub mod io;

/// Dispatches syscalls. Inputs are the raw register values, the return value
/// is the result of the syscall that is identified by the [`syscall`] argument.
// not unsafe because the caller can't do much about the argument validity anyways
pub fn dispatch_syscall(
    syscall: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> isize {
    let syscall = match TryInto::<Syscall>::try_into(syscall) {
        Ok(v) => v,
        Err(_) => return ENOSYS.into(),
    };

    unsafe {
        match syscall {
            Syscall::Access => dispatch_sys_access(arg1, arg2),
            Syscall::Read => dispatch_sys_read(arg1, arg2, arg3),
            Syscall::Write => dispatch_sys_write(arg1, arg2, arg3),
            Syscall::Open => dispatch_sys_open(arg1, arg2, arg3),
            Syscall::Close => dispatch_sys_close(arg1),
            _ => ENOSYS, // not implemented
        }
    }
    .into()
}

unsafe fn dispatch_sys_access(arg1: usize, arg2: usize) -> Errno {
    let ptr = arg1 as *const u8; // FIXME: check that `path` points to userspace data

    // FIXME: use PATH_MAX instead of hard coded 255
    let len = match strlen_s(ptr, 255) {
        None => return ENAMETOOLONG,
        Some(v) => v,
    };
    unsafe {
        let path = match core::str::from_utf8(from_raw_parts(ptr, len)) {
            Ok(v) => v,
            Err(_) => return EINVAL,
        };
        sys_access(path, AMode::from_bits_truncate(arg2))
    }
}

unsafe fn dispatch_sys_read(arg1: usize, arg2: usize, arg3: usize) -> Errno {
    unsafe { sys_read(arg1, from_raw_parts_mut(arg2 as *mut u8, arg3)) }
}

unsafe fn dispatch_sys_write(arg1: usize, arg2: usize, arg3: usize) -> Errno {
    unsafe { sys_write(arg1, from_raw_parts(arg2 as *const u8, arg3)) }
}

unsafe fn dispatch_sys_open(arg1: usize, arg2: usize, arg3: usize) -> Errno {
    let path = unsafe { *(arg1 as *const &str) };
    sys_open(path, arg2, arg3)
}

fn dispatch_sys_close(arg1: usize) -> Errno {
    sys_close(arg1)
}

fn strlen_s(ptr: *const u8, max: usize) -> Option<usize> {
    (0..max).find(|&i| unsafe { *ptr.add(i) } == 0)
}