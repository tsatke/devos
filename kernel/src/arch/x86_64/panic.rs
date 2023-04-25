use core::panic::PanicInfo;
use x86_64::instructions::hlt;

pub fn handle_panic(_info: &PanicInfo) -> ! {
    loop {
        hlt();
    }
}
