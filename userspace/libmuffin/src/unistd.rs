use kernel_abi::{SYS_CLOSE, SYS_GETCWD, SYS_LSEEK, SYS_READ, SYS_WRITE, SYS_WRITEV};
use libc::{c_char, c_int, size_t, ssize_t};

use crate::syscall::syscall3;
use crate::unimplemented_function;

#[unsafe(no_mangle)]
pub extern "C" fn close(fildes: c_int) -> i32 {
    unimplemented_function(SYS_CLOSE)
}

#[unsafe(no_mangle)]
pub extern "C" fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    unimplemented_function(SYS_GETCWD)
}

#[unsafe(no_mangle)]
pub extern "C" fn read(fildes: c_int, buf: *mut c_char, nbyte: size_t) -> ssize_t {
    syscall3(SYS_READ, fildes as usize, buf as usize, nbyte as usize)
}

#[unsafe(no_mangle)]
pub extern "C" fn write(fildes: c_int, buf: *const c_char, nbyte: size_t) -> ssize_t {
    syscall3(SYS_WRITE, fildes as usize, buf as usize, nbyte as usize)
}

#[unsafe(no_mangle)]
pub extern "C" fn writev(fildes: c_int, iov: *const libc::iovec, iovcnt: c_int) -> ssize_t {
    unimplemented_function(SYS_WRITEV)
}

#[unsafe(no_mangle)]
pub extern "C" fn lseek(fildes: c_int, offset: libc::off_t, whence: c_int) -> libc::off_t {
    unimplemented_function(SYS_LSEEK)
}
