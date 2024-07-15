#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use core::panic::PanicInfo;

use bootloader_api::{BootInfo, BootloaderConfig, entry_point};
use x86_64::structures::paging::PageTableFlags;

use kernel::{bootloader_config, kernel_init, serial_print, serial_println};
use kernel::io::vfs::vfs;
use kernel::mem::virt::MapAt;
use kernel::process::vmm;
use kernel::qemu::ExitCode;
use kernel_api::syscall::Stat;

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info).expect("kernel_init failed");

    serial_print!("test_read_file_via_vmo...");
    test_read_file_via_vmo();
    serial_println!("[ok]");

    kernel::qemu::exit(ExitCode::Success)
}

fn test_read_file_via_vmo() {
    let path = "/var/data/number_list_10000.txt";
    let node = vfs().open(path).expect("no such file");
    let mut stat = Stat::default();
    vfs().stat(&node, &mut stat).expect("unable to stat file");
    let size = stat.size as usize;

    let addr = vmm()
        .allocate_file_backed_vm_object(
            "test".to_string(),
            node,
            0,
            MapAt::Anywhere,
            size,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )
        .expect("unable to create file-based VmObject");

    // Access memory locations such that page faults are triggered that read the file content,
    // but the accesses are not page aligned. If we were to access all bytes once here,
    // all page-faults would trigger page-aligned reads. We don't want that in this test.
    for ptr in (addr..(addr + size)).step_by(2000) {
        unsafe { core::ptr::read_volatile(ptr.as_ptr::<u8>()) };
    }

    let slice = unsafe { core::slice::from_raw_parts(addr.as_ptr::<u8>(), size) };

    let actual = core::str::from_utf8(slice).expect("content is not valid UTF-8");
    let expected = include_str!("../../../os_disk/var/data/number_list_10000.txt");
    assert_eq!(actual, expected);

    // remove the vmobject from the process so that it gets dropped
    let _ = vmm().vm_objects().write().remove(&addr);
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
