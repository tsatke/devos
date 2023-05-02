#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::asm;
use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};

use kernel::arch::panic::handle_panic;
use kernel::mem::Size;
use kernel::{kernel_init, serial_println};

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

    if let Some(_) = ramdisk {
        serial_println!("got a ramdisk");
    }

    kernel_init(boot_info);

    let mut a: isize;
    unsafe {
        asm! {
            "mov rax, 0", // read syscall
            "mov rdi, 1", // fd
            "mov rsi, 5", // buffer address
            "mov rdx, 9", // num bytes
            "mov r8, 0", // unused arg4
            "int 0x80", // SYSCALL_INTERRUPT_INDEX
            "mov {}, rax",
            out(reg) a,
        };
    }
    serial_println!("a: {}", a);

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
