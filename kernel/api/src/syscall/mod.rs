use core::ops::BitAnd;

use bitflags::bitflags;
use num_enum::TryFromPrimitive;

pub use errno::*;

mod errno;

pub const SYSCALL_INTERRUPT_INDEX: usize = 0x80;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
#[repr(usize)]
pub enum Syscall {
    Read = 0,
    Write,
    Open,
    Close,
    Mmap,
    Access,
    Exit,
    Socket,
    Bind,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
#[repr(usize)]
pub enum SocketDomain {
    Unix = 0,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
#[repr(usize)]
pub enum SocketType {
    Stream = 0,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct FfiSockAddr {
    pub domain: SocketDomain,
    pub data: *const u8,
}

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
    pub struct FileMode: u32 {
        const S_IRWXU = 0o700;
        const S_IRUSR = 0o400;
        const S_IWUSR = 0o200;
        const S_IXUSR = 0o100;

        const S_IRWXG = 0o070;
        const S_IRGRP = 0o040;
        const S_IWGRP = 0o020;
        const S_IXGRP = 0o010;

        const S_IRWXO = 0o007;
        const S_IROTH = 0o004;
        const S_IWOTH = 0o002;
        const S_IXOTH = 0o001;

        const S_ISUID = 0o4000;
        const S_ISGID = 0o2000;
        const S_ISVTX = 0o1000;

        const S_IFMT = 0o170000;
        const S_IFSOCK = 0o140000;
        const S_IFLNK = 0o120000;
        const S_IFREG = 0o100000;
        const S_IFBLK = 0o060000;
        const S_IFDIR = 0o040000;
        const S_IFCHR = 0o020000;
        const S_IFIFO = 0o010000;
    }
}

macro_rules! is_mode {
    ($name:ident, $mask:expr) => {
        #[inline(always)]
        pub fn $name(mode: FileMode) -> bool {
            mode.bitand(FileMode::S_IFMT) == $mask
        }

        impl FileMode {
            pub fn $name(self) -> bool {
                $name(self)
            }
        }
    };
}

is_mode!(is_socket, FileMode::S_IFSOCK);
is_mode!(is_symlink, FileMode::S_IFLNK);
is_mode!(is_regular_file, FileMode::S_IFREG);
is_mode!(is_block_device, FileMode::S_IFBLK);
is_mode!(is_directory, FileMode::S_IFDIR);
is_mode!(is_char_device, FileMode::S_IFCHR);
is_mode!(is_fifo, FileMode::S_IFIFO);

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct Stat {
    pub dev: u64,
    pub ino: u64,
    pub mode: FileMode,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
    pub size: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub blksize: u64,
    pub blocks: u64,
}