#![no_std]
#![no_main]

use kernel::hpet::hpet;
use kernel::limine::BASE_REVISION;
use log::{error, info};
use x86_64::instructions::hlt;

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    info!("counter: {}", hpet().read().main_counter_value());
    info!("counter: {}", hpet().read().main_counter_value());
    info!("counter: {}", hpet().read().main_counter_value());
    info!("counter: {}", hpet().read().main_counter_value());
    info!("counter: {}", hpet().read().main_counter_value());

    info!("reached end of kernel_main");
    loop {
        hlt();
    }
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    let location = info.location().unwrap();
    error!(
        "kernel panicked at {}:{}:{}:\n{}",
        location.file(),
        location.line(),
        location.column(),
        info.message(),
    );
    loop {
        hlt();
    }
}
