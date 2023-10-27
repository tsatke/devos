use crate::io::path::Path;
use crate::io::vfs::vfs;
use crate::process::elf::ElfLoader;
use crate::{process, serial_println};
use alloc::string::ToString;
use alloc::vec;
use bitflags::bitflags;
use core::mem::transmute;
use elfloader::ElfBinary;
use kernel_api::syscall::{Errno, EIO, ENOENT, ENOSYS, OK};

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct AMode: usize {
        const F_OK = 0;
        const X_OK = 1;
        const W_OK = 2;
        const R_OK = 4;
    }
}
pub fn sys_access(path: impl AsRef<Path>, amode: AMode) -> Errno {
    if amode != AMode::F_OK {
        // TODO: support permissions
        return ENOSYS;
    }

    vfs().stat_path(path).map(|_| OK).unwrap_or(ENOENT)
}

pub fn sys_execve(path: impl AsRef<Path>, argv: &[&str], envp: &[&str]) -> Result<!, Errno> {
    serial_println!("sys_execve({:?}, {:?}, {:?})", path.as_ref(), argv, envp);

    let path = path.as_ref();

    let elf_data = {
        let file = vfs().open(path).map_err(|_| ENOENT)?;
        let stat = vfs().stat(&file).map_err(|_| EIO)?;
        let size = stat.size;
        let mut buf = vec![0_u8; size as usize];
        vfs().read(&file, &mut buf, 0).map_err(|_| EIO)?;
        buf
    };

    let mut loader = ElfLoader::default();
    let elf = ElfBinary::new(&elf_data).unwrap();
    elf.load(&mut loader).unwrap();
    let image = loader.into_inner();
    let entry = unsafe { image.as_ptr().add(elf.entry_point() as usize) };
    let entry_fn = unsafe { transmute(entry) };

    // execute the executable in the new task...
    process::spawn_task_in_current_process(path.to_string(), entry_fn);
    // ...and stop the current task
    unsafe { process::exit_current_task() }
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

pub fn sys_close(fd: usize) -> Errno {
    serial_println!("sys_close({})", fd);
    ENOSYS
}

pub fn sys_exit(status: usize) -> ! {
    serial_println!("sys_exit({})", status);
    process::exit();
}
