#![no_std]
#![no_main]
extern crate alloc;

use alloc::sync::Arc;
use core::ffi::c_void;
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

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info).expect("kernel_init failed");

    serial_print!("test_async_counter...");
    test_async_counter();
    serial_println!("[ok]");

    serial_print!("test_no_addressspace_lock...");
    test_no_addressspace_lock();
    serial_println!("[ok]");

    kernel::qemu::exit(ExitCode::Success)
}

fn test_async_counter() {
    let counter = Arc::new(AtomicU64::new(0));
    assert_eq!(0, counter.load(Relaxed));

    process::spawn_thread_in_current_process(
        "count1",
        Priority::Normal,
        count,
        Arc::into_raw(counter.clone()) as *mut c_void,
    );
    process::spawn_thread_in_current_process(
        "count2",
        Priority::Normal,
        count,
        Arc::into_raw(counter.clone()) as *mut c_void,
    );
    process::spawn_thread_in_current_process(
        "count3",
        Priority::Normal,
        count,
        Arc::into_raw(counter.clone()) as *mut c_void,
    );

    for _ in 0..20 {
        hlt(); // should be enough to get the functions scheduled 5 times each
    }

    assert_eq!(15, counter.load(Relaxed));

    assert_eq!(
        1,
        Arc::strong_count(&counter),
        "other threads should have dropped the Arcs once done"
    );
    assert_eq!(0, Arc::weak_count(&counter));
}

extern "C" fn count(cnt: *mut c_void) {
    let counter = unsafe { Arc::from_raw(cnt as *const AtomicU64) };
    assert!(
        Arc::strong_count(&counter) > 1,
        "Arc should be shared with at least the spawner"
    );
    for _ in 0..5 {
        counter.fetch_add(1, Relaxed);
    }
}

fn test_no_addressspace_lock() {
    let process = process::current();
    let guard = process.address_space().write();
    hlt(); // if the scheduler locks the address space in `reschedule`, this will deadlock
    drop(guard);
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
