#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::arch::asm;
use core::panic::PanicInfo;
use kernel::kernel_init;

#[cfg(not(test))]
entry_point!(kernel_main);

#[cfg(not(test))]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info);

    todo!()
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt", options(noreturn)) }
    }
}
