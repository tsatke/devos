#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use x86_64::instructions::hlt;
use x86_64::VirtAddr;

use graphics::{PrimitiveDrawing, Vec2};
use kernel::arch::panic::handle_panic;
use kernel::io::path::Path;
use kernel::io::vfs::vfs;
use kernel::mem::virt::{AllocationStrategy, VmObject};
use kernel::mem::Size;
use kernel::{kernel_init, process, screen, serial_println};
use vga::Color;

const KERNEL_STACK_SIZE: Size = Size::KiB(128);

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.page_table_recursive = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::Dynamic;
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

    ls("/bin");

    // let node = vfs().open("/bin/hello_world").unwrap();
    // let mut buf = [1u8; 10];
    // serial_println!("before: {:02x?}", buf);
    // vfs().read(&node, &mut buf, 0).unwrap();
    // serial_println!("after: {:02x?}", buf);
    //
    // process::spawn_task_in_current_process("vga_stuff", vga_stuff);
    // process::spawn_task_in_current_process("count_even", count_even);
    // process::spawn_task_in_current_process("count_odd", count_odd);
    //
    // let _other_process = process::create(process::current(), "other_process");

    // sys_execve("/bin/hello_world", &[], &[]).unwrap();

    let addr = VirtAddr::new(0x1111_0000_0000);
    let vm_object =
        VmObject::create_memory_backed(addr, 8192, AllocationStrategy::AllocateOnAccess).unwrap();
    process::current().write().add_vm_object(vm_object);

    unsafe { addr.as_mut_ptr::<u64>().write(0xdeadbeef) };
    unsafe {
        VirtAddr::new(addr.as_u64() + 4096)
            .as_mut_ptr::<u64>()
            .write(0xdeadbeef)
    };
    serial_println!("data: 0x{:x?}", unsafe { addr.as_ptr::<u64>().read() });

    panic!("kernel main returned")
}

fn ls(p: impl AsRef<Path>) {
    vfs().read_dir(p).unwrap().for_each(|e| {
        serial_println!("{:?}", e);
    })
}

extern "C" fn count_even() {
    for i in (0..10).step_by(2) {
        serial_println!("{}", i);
        hlt();
    }
}

extern "C" fn count_odd() {
    for i in (1..10).step_by(2) {
        serial_println!("{}", i);
        hlt();
    }
}

#[allow(dead_code)]
extern "C" fn vga_stuff() {
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
