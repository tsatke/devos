use crate::serial_println;

pub fn sys_read(fd: usize, buf: &mut [u8]) -> isize {
    serial_println!("sys_read({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    0
}

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    serial_println!("sys_write({}, {:#p}, {})", fd, buf.as_ptr(), buf.len());
    0
}

pub fn sys_open(path: &str, flags: usize, mode: usize) -> isize {
    serial_println!(
        "sys_open({:#p} ({}), {}, {})",
        path.as_ptr(),
        path,
        flags,
        mode
    );
    0
}

pub fn sys_close(fd: usize) -> isize {
    serial_println!("sys_close({})", fd);
    0
}
