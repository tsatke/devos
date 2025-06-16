//! fcntl.h

use core::slice::from_raw_parts;

use compiler_builtins::mem::strlen;
use kernel_abi::SYS_FCNTL;
use libc::{c_char, c_int};

use crate::syscall::Syscall;
use crate::unimplemented_function;

#[unsafe(no_mangle)]
#[allow(unused_mut)]
pub unsafe extern "C" fn open(path: *const c_char, oflag: c_int, mut varargs: ...) -> c_int {
    match Syscall::open(
        unsafe { from_raw_parts(path as *const u8, strlen(path)) },
        oflag as usize,
        0,
    ) {
        Ok(fd) => fd as c_int,
        Err(e) => {
            // Set errno based on the error
            crate::errno::set_errno(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
#[allow(unused_mut)]
pub unsafe extern "C" fn fcntl(fildes: c_int, cmd: c_int, mut varargs: ...) -> c_int {
    unimplemented_function(SYS_FCNTL)
}
