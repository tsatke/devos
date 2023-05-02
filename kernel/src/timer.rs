use crate::{process, serial_print};

pub fn notify_timer_interrupt() {
    unsafe { process::reschedule() }
}
