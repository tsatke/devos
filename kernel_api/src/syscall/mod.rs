use num_enum::TryFromPrimitive;

mod errno;

pub use errno::*;

pub const SYSCALL_INTERRUPT_INDEX: usize = 0x80;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
#[repr(usize)]
pub enum Syscall {
    Read = 0,
    Write,
    Open,
    Close,
    Stat,
    Fstat,
    Lstat,
    Poll,
    Lseek,
    Mmap,
    Mprotect,
    Munmap,
    Brk,
    RtSigAction,
    RtSigProcMask,
    Ioctl,
    Pread,
    Pwrite,
    Getcwd,
    Chdir,
    Dup,
    Pipe,
    Select,
    Flock,
    Ftruncate,
    Fsync,
    Fdatasync,
    Truncate,
    GetDents,
    GetPID,
    GetPPID,
    GetUID,
    GetEUID,
    GetGID,
    GetEGID,
    GetGroups,
    SetUID,
    SetGID,
    Access,
    Chown,
    Chmod,
    Link,
    Symlink,
    Unlink,
    Rename,
    Mkdir,
    Rmdir,
    GetTimeOfDay,
    ClockGetTime,
    Nanosleep,
    Exit,
}
