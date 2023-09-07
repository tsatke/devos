use crate::io::vfs::find;
use crate::serial_println;
use bitflags::bitflags;
use kernel_api::syscall::{Errno, ENOENT, ENOSYS, OK};

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct AMode: usize {
        const F_OK = 0;
        const X_OK = 1;
        const W_OK = 2;
        const R_OK = 4;
    }
}

pub fn sys_access(path: &str, amode: AMode) -> Errno {
    if amode != AMode::F_OK {
        // TODO: support permissions
        return ENOSYS;
    }

    if find(path).is_ok() {
        OK
    } else {
        ENOENT
    }
}

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
