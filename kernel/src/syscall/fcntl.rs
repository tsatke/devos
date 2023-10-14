use crate::io::path::Path;
use crate::serial_println;
use kernel_api::syscall::{Errno, ENOSYS};

// TODO: OpenFlags and Mode
pub fn sys_open(path: impl AsRef<Path>, flags: usize, mode: usize) -> Errno {
    serial_println!(
        "sys_open({:#p} ({}), {}, {})",
        path.as_ref().as_ptr(),
        path.as_ref(),
        flags,
        mode
    );
    ENOSYS
}
