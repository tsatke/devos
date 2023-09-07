#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::vec;
use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};

use kernel::arch::panic::handle_panic;
use kernel::mem::Size;
use kernel::process::syscall::io::sys_read;
use kernel::{kernel_init, serial_println};
use kernel_api::driver::BlockDevice;

const KERNEL_STACK_SIZE: Size = Size::KiB(32);

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.page_table_recursive = Some(Mapping::Dynamic);
    config.kernel_stack_size = KERNEL_STACK_SIZE.bytes() as u64;
    config
};

#[cfg(not(test))]
entry_point!(kernel_main, config = &CONFIG);

#[cfg(not(test))]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let ramdisk = boot_info
        .ramdisk_addr
        .into_option()
        .map(|v| (v as *const u8, boot_info.ramdisk_len as usize))
        .map(|(addr, len)| unsafe { from_raw_parts(addr, len) });

    if ramdisk.is_some() {
        serial_println!("got a ramdisk");
    }

    kernel_init(boot_info);

    let errno = sys_read(0, &mut [0]);
    serial_println!("result: {}", errno);

    let mut buf = vec![0_u8; 1030];
    let mut drive = ide::drives().next().unwrap().clone();
    drive.read_into(0, &mut buf).unwrap();
    serial_println!("read: {:02X?}", &buf[..512]);
    serial_println!("read: {:02X?}", &buf[512..1024]);
    serial_println!("read: {:02X?}", &buf[1024..]);

    panic!("kernel_main returned")
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!("kernel panicked: {}", info.message().unwrap());
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
