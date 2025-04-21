#![no_std]
#![no_main]

use x86_64::instructions::interrupts::int3;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        int3()
    }
}
