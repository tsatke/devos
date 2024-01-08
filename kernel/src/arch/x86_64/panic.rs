use core::panic::PanicInfo;

use x86_64::instructions::hlt;

use crate::process;

pub fn handle_panic(_info: &PanicInfo) -> ! {
    if process::current_thread().id() == &0_u64 {
        // FIXME: only for the kernel process, so pid=0?
        // we can't exit the kernel thread, so we just hlt forever
        loop {
            hlt();
        }
    } else {
        process::exit_thread();
    }
}
