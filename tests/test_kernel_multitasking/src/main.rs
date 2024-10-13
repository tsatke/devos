#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use x86_64::instructions::hlt;

use kernel::process::Priority;
use kernel::qemu::ExitCode;
use kernel::{bootloader_config, kernel_init, process, serial_print, serial_println};

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info).expect("kernel_init failed");

    serial_print!("test_async_counter...");
    test_counter();
    serial_println!("[ok]");

    serial_print!("test_no_addressspace_lock...");
    test_no_addressspace_lock();
    serial_println!("[ok]");

    kernel::qemu::exit(ExitCode::Success)
}

fn test_counter() {
    assert_eq!(0, COUNTER.load(Relaxed));

    process::spawn_thread_in_current_process("count1", Priority::Normal, count);
    process::spawn_thread_in_current_process("count2", Priority::Normal, count);
    process::spawn_thread_in_current_process("count3", Priority::Normal, count);

    for _ in 0..20 {
        hlt(); // should be enough to get the functions scheduled 5 times each
    }

    assert_eq!(15, COUNTER.load(Relaxed));
}

fn test_no_addressspace_lock() {
    let process = process::current();
    let guard = process.address_space().write();
    hlt(); // if the scheduler locks the address space in `reschedule`, this will deadlock
    drop(guard);
}

extern "C" fn count() {
    for _ in 0..5 {
        COUNTER.fetch_add(1, Relaxed);
    }
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
