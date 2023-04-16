#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::arch::asm;
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    todo!()
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt", options(noreturn)) }
    }
}
