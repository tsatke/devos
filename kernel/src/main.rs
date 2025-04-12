#![no_std]
#![no_main]
extern crate alloc;

use core::panic::PanicInfo;
use kernel::backtrace::Backtrace;
use kernel::limine::BASE_REVISION;
use kernel::mcore;
use log::error;
use x86_64::instructions::hlt;
use x86_64::instructions::interrupts::without_interrupts;

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    mcore::turn_idle()
}

#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    handle_panic(info);
    loop {
        hlt();
    }
}

fn handle_panic(info: &PanicInfo) {
    let location = info.location().unwrap();
    error!(
        "kernel panicked at {}:{}:{}:",
        location.file(),
        location.line(),
        location.column(),
    );
    error!("{}", info.message());
    without_interrupts(|| match Backtrace::try_capture() {
        Ok(bt) => {
            error!("stack backtrace:\n{bt}");
        }
        Err(e) => {
            error!("error capturing backtrace: {e:?}");
        }
    });
}
