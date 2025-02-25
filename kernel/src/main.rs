#![no_std]
#![no_main]

use jiff::Timestamp;
use kernel::hpet::hpet;
use kernel::limine::{BASE_REVISION, BOOT_TIME};
use log::{error, info};
use x86_64::instructions::hlt;

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    for _ in 0..10 {
        let counter = hpet().read().main_counter_value();
        let secs = BOOT_TIME.get_response().unwrap().boot_time().as_secs();
        let secs = secs + (counter / 1e9 as u64);
        let ts = Timestamp::new(secs as i64, (counter % 1_000_000_000) as i32).unwrap();
        info!("it is now {}", ts);
    }

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
