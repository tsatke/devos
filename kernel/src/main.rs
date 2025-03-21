#![no_std]
#![no_main]

use jiff::Timestamp;
use kernel::hpet::hpet;
use kernel::limine::{BASE_REVISION, BOOT_TIME};
use kernel::smp;
use log::{error, info};
use x86_64::instructions::hlt;

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    for _ in 0..5 {
        let counter = hpet().read().main_counter_value();
        let secs = BOOT_TIME.get_response().unwrap().timestamp().as_secs();
        let secs = secs + (counter / 1_000_000_000);
        let ts = Timestamp::new(
            i64::try_from(secs).expect("shouldn't have more seconds than i64::MAX"),
            (counter % 1_000_000_000) as i32,
        )
        .unwrap();
        info!("it is now {}", ts);
    }

    smp::start()
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
