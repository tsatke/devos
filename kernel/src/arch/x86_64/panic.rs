use crate::process;
use core::panic::PanicInfo;
use x86_64::instructions::hlt;

pub fn handle_panic(_info: &PanicInfo) -> ! {
    if process::current_task_id() == 0_u64 {
        // we can't exit the kernel task, so we just hlt forever
        loop {
            hlt();
        }
    } else {
        process::exit();
    }
}
