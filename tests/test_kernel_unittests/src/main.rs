#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::panic::PanicInfo;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};

use kernel::qemu::ExitCode;
use kernel::{bootloader_config, kernel_init, serial_print, serial_println};

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info);

    for test in kernel_test_framework::KERNEL_TESTS {
        serial_print!("test {}...", test.name);
        (test.test_fn)();
        serial_println!("[ok]")
    }

    kernel::qemu::exit(ExitCode::Success)
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!(
        "task '{}' panicked at {}:\n{}",
        kernel::process::current_task().name(),
        info.location().unwrap(),
        info.message().unwrap()
    );
    if let Some(location) = info.location() {
        serial_println!(
            "\tat {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    kernel::qemu::exit(ExitCode::Failed)
}
