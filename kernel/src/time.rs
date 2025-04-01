use crate::hpet::hpet;
use crate::BOOT_TIME_SECONDS;
use jiff::Timestamp;

pub trait TimestampExt {
    fn now() -> Self;
}

impl TimestampExt for Timestamp {
    fn now() -> Self {
        let counter = hpet().read().main_counter_value();
        let secs = BOOT_TIME_SECONDS.get().unwrap();
        let secs = secs + (counter / 1_000_000_000);
        Timestamp::new(
            i64::try_from(secs).expect("shouldn't have more seconds than i64::MAX"),
            (counter % 1_000_000_000) as i32,
        )
        .unwrap()
    }
}
