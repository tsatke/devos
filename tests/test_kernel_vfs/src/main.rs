#![no_std]
#![no_main]

use core::panic::PanicInfo;

use bootloader_api::{BootInfo, BootloaderConfig, entry_point};

use kernel::{bootloader_config, kernel_init, process, serial_println};
use kernel::qemu::ExitCode;
use kernel::syscall::sys_open;

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info).expect("kernel_init failed");

    let process = process::current();

    let hello_world = sys_open("/bin/hello_world", 0, 0).unwrap();
    let mut buf = [0u8; 13];
    assert_eq!(13, process.read(hello_world, &mut buf).unwrap());
    assert_eq!(&[127_u8, 69, 76, 70, 2, 1, 1, 0, 0, 0, 0, 0, 0], &buf);
    process.close_fd(hello_world).unwrap();

    kernel::qemu::exit(ExitCode::Success)
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!(
        "kernel panicked in pid={} ({}) tid={} ({}): {}",
        kernel::process::current().pid(),
        kernel::process::current().name(),
        kernel::process::current_thread().id(),
        kernel::process::current_thread().name(),
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

    kernel::qemu::exit(ExitCode::Failed)
}
