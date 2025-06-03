use crate::unimplemented_function;
use kernel_abi::SYS_SIGNAL;

#[unsafe(no_mangle)]
pub extern "C" fn signal(
    signum: libc::c_int,
    handler: extern "C" fn(libc::c_int) -> libc::c_int,
) -> extern "C" fn(libc::c_int) -> libc::c_int {
    unimplemented_function(SYS_SIGNAL)
}
