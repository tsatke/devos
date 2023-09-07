#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::vec;
use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use x86_64::structures::paging::{PageSize, Size4KiB};

use graphics::{PrimitiveDrawing, Vec2};
use kernel::arch::panic::handle_panic;
use kernel::mem::{MemoryManager, Size};
use kernel::process::syscall::io::sys_read;
use kernel::{kernel_init, screen, serial_println};
use kernel_api::driver::block::BlockDevice;
use vga::Color;

const KERNEL_STACK_SIZE: Size = Size::KiB(32);

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.page_table_recursive = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::FixedAddress(0xa0000);
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

    syscall_stuff();
    ide_stuff();
    vga_stuff();

    // try to produce an out-of-memory
    let mut mm = MemoryManager::lock();
    let count = 1_u64 << 15;
    serial_println!(
        "trying to allocate (and deallocate) {} MiB of physical memory",
        count * Size4KiB::SIZE / 1024 / 1024
    );
    let tsc_start = unsafe { core::arch::x86_64::_rdtsc() };
    for i in 0..count {
        let res = mm.allocate_frame();
        if res.is_none() {
            serial_println!("out of physical memory after {} iterations", i);
            break;
        }
        mm.deallocate_frame(res.unwrap());
    }
    let tsc_end = unsafe { core::arch::x86_64::_rdtsc() };
    let avg = (tsc_end - tsc_start) / count;
    serial_println!(
        "average cpu cycles per physical frame allocation and deallocation: {}",
        avg
    );

    panic!("kernel_main returned")
}

fn syscall_stuff() {
    serial_println!("calling sys_read...");
    let errno = sys_read(0, &mut [0]);
    serial_println!("result: {}", errno);
}

fn ide_stuff() {
    const CNT: usize = 1030;
    serial_println!("reading the first {} bytes from the boot drive...", CNT);
    let mut buf = vec![0_u8; CNT];
    let mut drive = ide::drives().next().unwrap().clone();
    drive.read_into(0, &mut buf).unwrap();
    serial_println!("read: {:02X?}", &buf[..512]);
    serial_println!("read: {:02X?}", &buf[512..1024]);
    serial_println!("read: {:02X?}", &buf[1024..]);
}

fn vga_stuff() {
    let mut vga = screen::lock();

    // white screen
    vga.clear_screen(Color::White);

    // mesh
    for l in (80..720).step_by(20) {
        for r in (80..720).step_by(20) {
            vga.draw_line(Vec2 { x: 680, y: l }, Vec2 { x: 1200, y: r }, Color::Black);
        }
    }

    // triangle
    vga.fill_triangle(
        Vec2 { x: 600, y: 700 },
        Vec2 { x: 200, y: 600 },
        Vec2 { x: 400, y: 500 },
        Color::Black,
    );

    // colors
    vga.fill_rect(Vec2 { x: 10, y: 10 }, Vec2 { x: 60, y: 60 }, Color::Red);
    vga.fill_rect(Vec2 { x: 70, y: 10 }, Vec2 { x: 120, y: 60 }, Color::Green);
    vga.fill_rect(Vec2 { x: 130, y: 10 }, Vec2 { x: 180, y: 60 }, Color::Blue);

    // gradient
    let start_point = Vec2 { x: 100, y: 200 };
    let end_point = Vec2 { x: 300, y: 400 };

    let width = end_point.x - start_point.x + 1;
    let height = end_point.y - start_point.y + 1;

    for y in 0..height {
        for x in 0..width {
            let r = (x as f32 / width as f32 * 255.0) as u32;
            let b = (y as f32 / height as f32 * 255.0) as u32;
            let color = Color::Other(0xff_ff_ff - (r << 16) - b);

            vga.set_pixel(
                Vec2 {
                    x: start_point.x + x,
                    y: start_point.y + y,
                },
                color,
            );
        }
    }
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
