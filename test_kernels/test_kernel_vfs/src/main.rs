#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::panic::PanicInfo;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};

use kernel::io::vfs::vfs;
use kernel::qemu::ExitCode;
use kernel::{bootloader_config, kernel_init, serial_println};

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info);

    let hello_world = vfs().open("/bin/hello_world").unwrap();
    let mut buf = [0u8; 13];
    assert_eq!(13, vfs().read(&hello_world, &mut buf, 0).unwrap());
    assert_eq!(&[127_u8, 69, 76, 70, 2, 1, 1, 0, 0, 0, 0, 0, 0], &buf);
    drop(hello_world); // this shouldn't panic

    kernel::qemu::exit(ExitCode::Success)
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!(
        "kernel panicked in pid={} ({}) tid={} ({}): {}",
        kernel::process::current().process_id(),
        kernel::process::current().name(),
        kernel::process::current_task().task_id(),
        kernel::process::current_task().name(),
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
