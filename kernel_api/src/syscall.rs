use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive)]
#[repr(usize)]
pub enum Syscall {
    Read = 0,
    Write = 1,
    Open = 2,
}
