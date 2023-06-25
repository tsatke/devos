#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;
extern crate kernel;

use alloc::format;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use itertools::Itertools;
use kernel::qemu::{exit, ExitCode};
use kernel::{bootloader_config, kernel_init, serial_print, serial_println};
use kernel_test_framework::KERNEL_TESTS;

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info);

    serial_println!("running {} tests", KERNEL_TESTS.len());
    for t in KERNEL_TESTS
        .iter()
        .sorted_by(|l, r| l.test_location.module.cmp(r.test_location.module))
    {
        let location = &t.test_location;
        let display_name = format!("{}::{}", location.module, t.name);
        serial_print!("test {} ... ", display_name);
        (t.test_fn)();
        serial_println!("\u{001b}[32m{}\u{001b}[0m", "ok");
    }
    serial_println!("done");

    exit(ExitCode::Success)
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!("\u{001b}[31m{}\u{001b}[0m", "FAILED");
    serial_println!("test panicked: {}", info.message().unwrap());
    if let Some(location) = info.location() {
        serial_println!(
            "\tat {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
    exit(ExitCode::Failed)
}
