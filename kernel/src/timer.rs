use crate::process;

pub fn notify_timer_interrupt() {
    unsafe { process::reschedule() }
}
