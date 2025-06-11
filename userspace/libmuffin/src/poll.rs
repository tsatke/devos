use kernel_abi::SYS_POLL;
use libc::c_int;

use crate::unimplemented_function;

#[unsafe(no_mangle)]
pub extern "C" fn poll(fds: *mut libc::pollfd, nfds: libc::nfds_t, timeout: c_int) -> c_int {
    unimplemented_function(SYS_POLL)
}
