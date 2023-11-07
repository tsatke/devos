use core::cmp::Ordering;
use core::ops::Add;

/// A 2D vector that may be used for points on the screen or sizes.
///
/// # Ord
/// A vector `v1` is smaller than `v2` if it is higher and more left on
/// the screen.
/// * `(0, 0) < (10, 10)`
/// * `(0, 0) < (0, 10)`
/// * `(0, 0) < (10, 0)`
/// * `(10, 0) < (0, 10)`
///
/// # Eq
/// The [`Eq`] implementation is trivial.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Vec2 {
    pub x: usize,
    pub y: usize,
}

impl Vec2 {
    /// The zero vector `(0, 0)`.
    pub const ZERO: Vec2 = Vec2 { x: 0, y: 0 };
}

impl From<(usize, usize)> for Vec2 {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<bresenham::Point> for Vec2 {
    fn from(value: bresenham::Point) -> Self {
        Self {
            x: value.0 as usize,
            y: value.1 as usize,
        }
    }
}

impl From<Vec2> for bresenham::Point {
    fn from(value: Vec2) -> Self {
        (value.x as isize, value.y as isize)
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl PartialOrd for Vec2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Vec2 {
    fn cmp(&self, other: &Self) -> Ordering {
        let res = self.y.cmp(&other.y);
        match res {
            Ordering::Equal => self.x.cmp(&other.x),
            _ => res,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ord() {
        assert_eq!(Vec2 { x: 0, y: 0 }, Vec2 { x: 0, y: 0 });
        assert!(Vec2 { x: 0, y: 0 } < Vec2 { x: 10, y: 10 });
        assert!(Vec2 { x: 10, y: 0 } < Vec2 { x: 0, y: 10 });
    }
}
