#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use core::slice::from_raw_parts;
use core::time::Duration;
use kernel::arch::panic::handle_panic;
use kernel::process::{change_thread_priority, Priority, Process};
use kernel::time::Instant;
use kernel::{bootloader_config, kernel_init, process, serial_println};
use x86_64::instructions::hlt;

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let ramdisk = boot_info
        .ramdisk_addr
        .into_option()
        .map(|v| (v as *const u8, boot_info.ramdisk_len as usize))
        .map(|(addr, len)| unsafe { from_raw_parts(addr, len) });

    if ramdisk.is_some() {
        serial_println!("got a ramdisk");
    }

    kernel_init(boot_info).expect("kernel_init failed");

    let start = Instant::now();
    let _ = Process::spawn_from_executable(
        process::current(),
        "/bin/window_server",
        Priority::Realtime,
        0.into(),
        0.into(),
    );
    serial_println!("window_server spawned in {:?}", Instant::now() - start);

    change_thread_priority(Priority::Low);

    panic!("kernel_main returned");
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!(
        "kernel panicked in pid={} ({}) tid={} ({}): {}",
        process::current().pid(),
        process::current().name(),
        process::current_thread().id(),
        process::current_thread().name(),
        info.message()
    );
    if let Some(location) = info.location() {
        serial_println!(
            "\tat {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    handle_panic(info)
}
