use num_enum::TryFromPrimitive;

pub const SYSCALL_INTERRUPT_INDEX: usize = 0x80;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
#[repr(usize)]
pub enum Syscall {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
}
