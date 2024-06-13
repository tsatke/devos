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
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
#[repr(usize)]
pub enum SocketDomain {
    Unix,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
#[repr(usize)]
pub enum SocketType {
    Stream,
}