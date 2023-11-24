#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::string::ToString;
use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};

use graphics::{PrimitiveDrawing, Vec2};
use kernel::arch::panic::handle_panic;
use kernel::process::process_tree;
use kernel::syscall::sys_execve;
use kernel::{bootloader_config, kernel_init, process, screen, serial_println};
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

    process::spawn_task_in_current_process("vga_stuff", vga_stuff);
    process::spawn_task_in_current_process("hello_world", hello_world);

    for _ in 0..5 {
        let new_process = process::create(process::current().clone(), "new_process".to_string());
        process::spawn_task("greet", &new_process, greet);
    }

    process_tree().read().dump();

    panic!("kernel_main returned");
}

extern "C" fn greet() {
    serial_println!("hello from pid {}", process::current().process_id());
}

extern "C" fn hello_world() {
    sys_execve("/bin/hello_world", &[], &[]).unwrap();
}

extern "C" fn vga_stuff() {
    if !screen::vga_initialized() {
        serial_println!("screen not initialized, skipping graphics");
        return;
    }

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

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!(
        "kernel panicked in pid={} ({}) tid={} ({}): {}",
        process::current().process_id(),
        process::current().name(),
        process::current_task().task_id(),
        process::current_task().name(),
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

    handle_panic(info)
}
