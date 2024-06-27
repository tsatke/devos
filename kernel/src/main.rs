#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::vec;
use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::{BootInfo, BootloaderConfig, entry_point};

use graphics::PrimitiveDrawing;
use kernel::{bootloader_config, kernel_init, process, screen, serial_println};
use kernel::arch::panic::handle_panic;
use vga::Color;

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

    // process::spawn_thread_in_current_process("vga_stuff", vga_stuff);
    //
    // let _ = Process::spawn_from_executable(
    //     process::current(),
    //     "/bin/hello_world",
    //     0.into(),
    //     0.into(),
    // );
    //
    // let _ = Process::spawn_from_executable(
    //     process::current(),
    //     "/bin/window_server",
    //     0.into(),
    //     0.into(),
    // );

    let p = process::current();

    let fd = p.open_file("/var/data/hello.txt").unwrap();
    let data = vec![b'X'; 1719];
    loop
    {
        let written = p.write(fd, &data).unwrap();
        let sz = p.stat(fd).unwrap().size;
        serial_println!("written: {} (size is now {})", written, sz);
        if sz as usize + data.len() >= 12288 {
            break;
        }
    }
    p.close_fd(fd).unwrap();

    let mut buf = [0_u8; 256];
    let fd = p.open_file("/var/data/hello.txt").unwrap();
    let read = p.read(fd, &mut buf).unwrap();
    serial_println!("read: {}", read);
    buf.iter().for_each(|v| assert_eq!(v, &b'X'));
    p.close_fd(fd).unwrap();

    panic!("kernel_main returned");
}

extern "C" fn vga_stuff() {
    if !screen::vga_initialized() {
        serial_println!("screen not initialized, skipping graphics");
        return;
    }

    let mut vga = screen::lock();

    // white screen
    vga.clear_screen(Color::White);
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
