use core::time::Duration;
use derive_more::Constructor;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Constructor)]
pub struct Instant {
    nanos: u64,
}

impl Instant {
    pub fn checked_duration_since(&self, earlier: &Self) -> Option<Duration> {
        if self > earlier {
            Some(*self - *earlier)
        } else {
            None
        }
    }

    pub fn duration_since(&self, earlier: Self) -> Duration {
        Duration::from_nanos(self.nanos.saturating_sub(earlier.nanos))
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Instant::new(self.nanos + rhs.as_nanos() as u64)
    }
}

impl core::ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Instant::new(self.nanos - rhs.as_nanos() as u64)
    }
}

impl core::ops::Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration::from_nanos(self.nanos - rhs.nanos)
    }
}

impl core::ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}
