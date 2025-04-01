#![no_std]
#![no_main]

use kernel::limine::BASE_REVISION;
use kernel::{mcore, now};
use log::{error, info};
use x86_64::instructions::hlt;

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    for _ in 0..5 {
        let ts = now();
        info!("it is now {ts}");
    }

    mcore::start()
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
