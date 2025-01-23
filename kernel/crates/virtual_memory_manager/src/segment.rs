use x86_64::VirtAddr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Segment {
    pub start: VirtAddr,
    pub len: u64,
}

impl Segment {
    #[must_use]
    pub const fn new(start: VirtAddr, len: u64) -> Self {
        Self { start, len }
    }
}
