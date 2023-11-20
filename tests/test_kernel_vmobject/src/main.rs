#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::string::ToString;
use core::panic::PanicInfo;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use x86_64::structures::paging::PageTableFlags;

use kernel::mem::virt::{AllocationStrategy, MapAt};
use kernel::process::vmm;
use kernel::qemu::ExitCode;
use kernel::{bootloader_config, kernel_init, serial_print, serial_println};

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info).expect("kernel_init failed");

    serial_print!("test_memory_backed_allocate_now...");
    test_memory_backed(AllocationStrategy::AllocateNow);
    serial_println!("[ok]");

    serial_print!("test_memory_backed_allocate_on_access...");
    test_memory_backed(AllocationStrategy::AllocateOnAccess);
    serial_println!("[ok]");

    kernel::qemu::exit(ExitCode::Success)
}

fn test_memory_backed(allocation_strategy: AllocationStrategy) {
    let addr = vmm()
        .allocate_memory_backed_vmobject(
            "test".to_string(),
            MapAt::Anywhere,
            8192,
            allocation_strategy,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )
        .expect("unable to create VmObject");

    unsafe {
        let ptr1 = addr.as_mut_ptr::<u64>();
        assert_eq!(0, ptr1.read());
        ptr1.write(0xdeadcafebeefbabe);
        assert_eq!(0xdeadcafebeefbabe, ptr1.read());

        let ptr2 = (addr + 4096_usize).as_mut_ptr::<u64>();
        assert_eq!(0, ptr2.read());
        ptr2.write(0x1234567822447799);
        assert_eq!(0x1234567822447799, ptr2.read());
    }

    // remove the vmobject from the process so that it gets dropped
    let _ = vmm().vm_objects().write().remove(&addr);
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!(
        "kernel panicked in pid={} ({}) tid={} ({}): {}",
        kernel::process::current().process_id(),
        kernel::process::current().name(),
        kernel::process::current_task().task_id(),
        kernel::process::current_task().name(),
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

    kernel::qemu::exit(ExitCode::Failed)
}
