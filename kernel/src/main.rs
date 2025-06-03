#![no_std]
#![no_main]
extern crate alloc;

use core::panic::PanicInfo;
use kernel::limine::BASE_REVISION;
use kernel::mcore;
use kernel::mcore::mtask::process::Process;
use kernel_vfs::path::AbsolutePath;
use log::{error, info};
use x86_64::instructions::hlt;

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    let proc = Process::create_from_executable(
        Process::root(),
        AbsolutePath::try_new("/bin/sandbox").unwrap(),
    )
    .unwrap();
    info!("have process pid={}", proc.pid());

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

    #[cfg(feature = "backtrace")]
    match kernel::backtrace::Backtrace::try_capture() {
        Ok(bt) => {
            error!("stack backtrace:\n{bt}");
        }
        Err(e) => {
            error!("error capturing backtrace: {e:?}");
        }
    }
}
