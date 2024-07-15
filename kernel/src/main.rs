#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::{BootInfo, BootloaderConfig, entry_point};

use kernel::{bootloader_config, kernel_init, process, serial_println};
use kernel::arch::panic::handle_panic;
use kernel::io::vfs::vfs;
use kernel::process::{change_thread_priority, Priority, Process};
use kernel_api::syscall::Stat;

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

    let _ = Process::spawn_from_executable(
        process::current(),
        "/bin/hello_world",
        Priority::Normal,
        0.into(),
        0.into(),
    );

    let _ = Process::spawn_from_executable(
        process::current(),
        "/bin/window_server",
        Priority::Realtime,
        0.into(),
        0.into(),
    );

    for p in ["/var", "/var/data", "/dev", "/bin"] {
        serial_println!("listing {}", p);
        vfs().read_dir(p)
            .unwrap()
            .into_iter()
            .filter(|v| v.name != "." && v.name != "..") // FIXME: we currently can't handle . and .. during path resolution
            .for_each(|entry| {
                let mut stat = Stat::default();
                vfs().stat_path(format!("{}/{}", p, entry.name), &mut stat).unwrap();
                serial_println!("{} {}", stat.mode, entry.name);
            });
    }

    change_thread_priority(Priority::Low);

    panic!("kernel_main returned");
}

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
