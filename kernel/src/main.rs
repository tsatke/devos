#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::vec;
use core::arch::asm;
use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use elfloader::ElfBinary;

use graphics::{PrimitiveDrawing, Vec2};
use kernel::arch::panic::handle_panic;
use kernel::io::vfs::InodeBase;
use kernel::io::vfs::{find, Inode};
use kernel::mem::Size;
use kernel::process::elf::ElfLoader;
use kernel::syscall::io::{sys_access, AMode};
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

    process::spawn_task("vga_stuff", vga_stuff);

    serial_println!("sys_access /dev: {:?}", sys_access("/dev", AMode::F_OK));
    ls("/");
    ls("/dev");
    ls("/mnt");

    process::spawn_task("elf_stuff", elf_stuff);

    panic!("kernel_main returned")
}

#[no_mangle]
extern "C" fn elf_stuff() {
    serial_println!(
        "sys_access /hello_world: {:?}",
        sys_access("/hello_world", AMode::F_OK)
    );

    let elf_data = {
        let file = find("/hello_world").unwrap().as_file().unwrap();
        let guard = file.read();
        let size = guard.size();
        let mut buf = vec![0_u8; size as usize];
        guard.read_at(0, &mut buf).unwrap();
        buf
    };

    let mut loader = ElfLoader::default();
    let elf = ElfBinary::new(&elf_data).unwrap();
    elf.load(&mut loader).unwrap();
    let image = loader.into_inner();
    let entry = unsafe { image.as_ptr().add(elf.entry_point() as usize) };
    serial_println!("jumping to entry: {:#p}", entry);
    unsafe {
        asm!("jmp {}", in(reg) entry);
    }
}

fn ls(path: &str) {
    let root = find(path).ok().and_then(|inode| inode.as_dir()).unwrap();
    let guard = root.read();
    let children = guard.children().unwrap();
    serial_println!("ls '{}'", path);
    for child in children.iter() {
        let indicator = match child {
            Inode::Dir(_) => "d",
            Inode::File(_) => "f",
            Inode::BlockDevice(_) => "b",
            Inode::CharacterDevice(_) => "c",
            Inode::Symlink(_) => "l",
        };
        serial_println!("  {} {}", indicator, child.name());
    }
}

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
