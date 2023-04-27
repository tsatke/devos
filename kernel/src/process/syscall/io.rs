use crate::serial_println;

pub fn sys_read(fd: usize, buf: *mut u8, count: usize) -> isize {
    serial_println!("sys_read({}, {:#p}, {})", fd, buf, count);
    let buf = unsafe { core::slice::from_raw_parts_mut(buf, count) };
    7
}

pub fn sys_write(fd: usize, buf: *mut u8, count: usize) -> isize {
    serial_println!("sys_write({}, {:#p}, {})", fd, buf, count);
    let buf = unsafe { core::slice::from_raw_parts(buf, count) };
    8
}

// TODO: figure out how to pass strings, because we're certainly not using 0 terminated ones
pub fn sys_open(path: /* this right here */ *const u8, flags: usize, mode: usize) -> isize {
    serial_println!("sys_open({:#p}, {}, {})", path, flags, mode);
    9
}
