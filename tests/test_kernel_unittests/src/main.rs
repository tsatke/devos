#![no_std]
#![no_main]

use core::panic::PanicInfo;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use kernel::qemu::ExitCode;
use kernel::{bootloader_config, kernel_init, serial_print, serial_println};
use log::error;

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info).expect("kernel_init failed");

    for test in kernel_test_framework::KERNEL_TESTS {
        serial_print!("test {}...", test.name);
        (test.test_fn)();
        serial_println!("[ok]")
    }

    kernel::qemu::exit(ExitCode::Success)
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    error!("[failed]");
    error!(
        "thread '{}' panicked at {}:\n{}",
        kernel::process::current_thread().name(),
        info.location().unwrap(),
        info.message()
    );
    if let Some(location) = info.location() {
        error!(
            "\tat {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    kernel::qemu::exit(ExitCode::Failed)
}
