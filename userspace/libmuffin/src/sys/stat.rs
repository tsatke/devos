use crate::unimplemented_function;
use kernel_abi::{SYS_FSTAT, SYS_STAT};
use libc::{c_char, c_int};

#[unsafe(no_mangle)]
pub extern "C" fn stat(path: *const c_char, buf: *mut libc::stat) -> c_int {
    unimplemented_function(SYS_STAT)
}

#[unsafe(no_mangle)]
pub extern "C" fn fstat(fd: c_int, buf: *mut libc::stat) -> c_int {
    unimplemented_function(SYS_FSTAT)
}
