use core::slice::{from_raw_parts, from_raw_parts_mut};

use kernel_abi::{SYS_CLOSE, SYS_GETCWD, SYS_LSEEK, SYS_WRITEV};
use libc::{c_char, c_int, size_t, ssize_t};

use crate::errno::set_errno;
use crate::syscall::Syscall;
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
    match Syscall::read(fildes as usize, unsafe {
        from_raw_parts_mut(buf as *mut u8, nbyte)
    }) {
        Ok(bytes_read) => bytes_read as ssize_t,
        Err(e) => {
            set_errno(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn write(fildes: c_int, buf: *const c_char, nbyte: size_t) -> ssize_t {
    match Syscall::write(fildes as usize, unsafe {
        from_raw_parts(buf as *const u8, nbyte)
    }) {
        Ok(bytes_written) => bytes_written as ssize_t,
        Err(e) => {
            set_errno(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn writev(fildes: c_int, iov: *const libc::iovec, iovcnt: c_int) -> ssize_t {
    unimplemented_function(SYS_WRITEV)
}

#[unsafe(no_mangle)]
pub extern "C" fn lseek(fildes: c_int, offset: libc::off_t, whence: c_int) -> libc::off_t {
    unimplemented_function(SYS_LSEEK)
}
