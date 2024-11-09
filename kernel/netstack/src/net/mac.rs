#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MacAddr([u8; 6]);

impl MacAddr {
    pub const BROADCAST: Self = Self([0xFF; 6]);

    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }
}
