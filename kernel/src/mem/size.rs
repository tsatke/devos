use core::cmp::Ordering;
use core::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum Size {
    KiB(usize),
    MiB(usize),
    GiB(usize),
    TiB(usize),
}

impl Size {
    pub const fn bytes(self) -> usize {
        match self {
            Size::KiB(x) => x * 1024,
            Size::MiB(x) => x * Size::KiB(1024).bytes(),
            Size::GiB(x) => x * Size::MiB(1024).bytes(),
            Size::TiB(x) => x * Size::GiB(1024).bytes(),
        }
    }
}

impl PartialEq for Size {
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}

impl Eq for Size {}

impl PartialOrd for Size {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.bytes().cmp(&other.bytes()))
    }
}

impl Ord for Size {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Size::KiB(v) => write!(f, "{} KiB", v),
            Size::MiB(v) => write!(f, "{} MiB", v),
            Size::GiB(v) => write!(f, "{} GiB", v),
            Size::TiB(v) => write!(f, "{} TiB", v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_ord() {
        let _1k = Size::KiB(1);
        let _2k = Size::KiB(2);
        let _1m = Size::MiB(1);
        let _2m = Size::MiB(2);
        let _1g = Size::GiB(1);
        let _2g = Size::GiB(2);
        let _1t = Size::TiB(1);
        let _2t = Size::TiB(2);
        assert!(_1k < _2k);
        assert!(_2k < _1m);
        assert!(_1m < _2m);
        assert!(_2m < _1g);
        assert!(_1g < _2g);
        assert!(_2g < _1t);
        assert!(_1t < _2t);
    }

    #[test_case]
    fn test_eq() {
        assert_eq!(Size::KiB(1), Size::KiB(1));
        assert_eq!(Size::MiB(1), Size::MiB(1));
        assert_eq!(Size::GiB(1), Size::GiB(1));
        assert_eq!(Size::TiB(1), Size::TiB(1));

        assert_eq!(Size::KiB(1024), Size::MiB(1));
        assert_eq!(Size::MiB(1024), Size::GiB(1));
        assert_eq!(Size::GiB(1024), Size::TiB(1));
    }
}
