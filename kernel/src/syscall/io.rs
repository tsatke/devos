use crate::serial_println;
use kernel_api::syscall::{Errno, ENOSYS};

pub fn sys_read(fd: usize, buf: &mut [u8]) -> Errno {
    serial_println!("sys_read({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    buf[0] = 1;
    1.into()
}

pub fn sys_write(fd: usize, buf: &[u8]) -> Errno {
    serial_println!("sys_write({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    ENOSYS
}

pub fn sys_open(path: &str, flags: usize, mode: usize) -> Errno {
    serial_println!(
        "sys_open({:#p} ({}), {}, {})",
        path.as_ptr(),
        path,
        flags,
        mode
    );
    ENOSYS
}

pub fn sys_close(fd: usize) -> Errno {
    serial_println!("sys_close({})", fd);
    ENOSYS
}
