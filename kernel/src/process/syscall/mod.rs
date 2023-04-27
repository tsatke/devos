use core::slice::{from_raw_parts, from_raw_parts_mut};
use kernel_api::syscall::Syscall;

use crate::process::syscall::io::*;

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
) -> isize {
    let syscall = TryInto::<Syscall>::try_into(syscall);
    if syscall.is_err() {
        return -1;
    }
    let syscall = syscall.unwrap();

    unsafe {
        match syscall {
            Syscall::Read => dispatch_sys_read(arg1, arg2, arg3),
            Syscall::Write => dispatch_sys_write(arg1, arg2, arg3),
            Syscall::Open => dispatch_sys_open(arg1, arg2, arg3),
            Syscall::Close => dispatch_sys_close(arg1),
        }
    }
}

unsafe fn dispatch_sys_read(arg1: usize, arg2: usize, arg3: usize) -> isize {
    unsafe { sys_read(arg1, from_raw_parts_mut(arg2 as *mut u8, arg3)) }
}

unsafe fn dispatch_sys_write(arg1: usize, arg2: usize, arg3: usize) -> isize {
    unsafe { sys_write(arg1, from_raw_parts(arg2 as *const u8, arg3)) }
}

unsafe fn dispatch_sys_open(arg1: usize, arg2: usize, arg3: usize) -> isize {
    let path = unsafe { *(arg1 as *const &str) };
    sys_open(path, arg2, arg3)
}

fn dispatch_sys_close(arg1: usize) -> isize {
    sys_close(arg1)
}
