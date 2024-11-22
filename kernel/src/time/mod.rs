use crate::driver::hpet::hpet;
pub use instant::*;

mod instant;

pub trait Clock {
    fn now() -> Instant;
}

pub struct HpetClock;

impl Clock for HpetClock {
    fn now() -> Instant {
        const FEMTOSECONDS_PER_NANOSECOND: u128 = 1_000_000;

        let hpet = hpet().read();
        let period = hpet.period_femtoseconds() as u128;
        let ticks = hpet.main_counter_value() as u128;
        let nanos = (ticks * period) / FEMTOSECONDS_PER_NANOSECOND;

        Instant::new(u64::try_from(nanos).unwrap())
    }
}