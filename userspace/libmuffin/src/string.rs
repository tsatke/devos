use kernel_abi::SYS_STRERROR_R;
use libc::{c_char, c_int, c_void, size_t};

use crate::unimplemented_function;

#[unsafe(no_mangle)]
pub extern "C" fn memcpy(dest: *mut c_char, src: *const c_char, n: size_t) -> *mut c_void {
    unsafe { compiler_builtins::mem::memcpy(dest.cast(), src.cast(), n) }.cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn memset(dest: *mut c_char, val: c_int, n: size_t) -> *mut c_void {
    unsafe { compiler_builtins::mem::memset(dest.cast(), val, n) }.cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn memcmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
    unsafe { compiler_builtins::mem::memcmp(s1.cast(), s2.cast(), n) }
}

#[unsafe(no_mangle)]
pub extern "C" fn memmove(dest: *mut c_char, src: *const c_char, n: size_t) -> *mut c_void {
    unsafe { compiler_builtins::mem::memmove(dest.cast(), src.cast(), n) }.cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn strlen(s: *const c_char) -> size_t {
    unsafe { compiler_builtins::mem::strlen(s.cast()) }
}

#[unsafe(no_mangle)]
pub extern "C" fn strerror_r(errnum: c_int) -> *const c_char {
    unimplemented_function(SYS_STRERROR_R)
}
