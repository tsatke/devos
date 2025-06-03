use crate::unimplemented_function;
use kernel_abi::{
    SYS_ABORT, SYS_CALLOC, SYS_FREE, SYS_GETENV, SYS_MALLOC, SYS_POSIX_MEMALIGN, SYS_REALLOC,
    SYS_REALPATH,
};
use libc::{c_char, c_int, c_void, size_t};

#[unsafe(no_mangle)]
pub extern "C" fn getenv(name: *const c_char) -> *const c_char {
    unimplemented_function(SYS_GETENV)
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: size_t) -> *mut c_void {
    unimplemented_function(SYS_MALLOC)
}

#[unsafe(no_mangle)]
pub extern "C" fn free(ptr: *mut c_void) {
    unimplemented_function(SYS_FREE)
}

#[unsafe(no_mangle)]
pub extern "C" fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    unimplemented_function(SYS_REALLOC)
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(nelem: size_t, elsize: size_t) -> *mut c_void {
    unimplemented_function(SYS_CALLOC)
}

#[unsafe(no_mangle)]
pub extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    alignment: size_t,
    size: size_t,
) -> c_int {
    unimplemented_function(SYS_POSIX_MEMALIGN)
}

#[unsafe(no_mangle)]
pub extern "C" fn abort() -> ! {
    unimplemented_function(SYS_ABORT)
}

#[unsafe(no_mangle)]
pub extern "C" fn realpath(path: *const c_char, resolved: *mut c_char) -> *mut c_char {
    unimplemented_function(SYS_REALPATH)
}
