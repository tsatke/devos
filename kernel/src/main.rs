#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};

use kernel::arch::panic::handle_panic;
use kernel::driver::pci;
use kernel::driver::pci::{PciDeviceClass, PciStandardHeaderDevice, SerialBusSubClass};
use kernel::driver::xhci::Xhci;
use kernel::process::{change_thread_priority, Priority, Process};
use kernel::{bootloader_config, kernel_init, process, serial_println};

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

    let _ = Process::spawn_from_executable(
        process::current(),
        "/bin/window_server",
        Priority::Realtime,
        0.into(),
        0.into(),
    );

    let xhci = pci::devices()
        .filter(|d| matches!(d.class(), PciDeviceClass::SerialBusController(SerialBusSubClass::USBController)) && d.prog_if() == 0x30)
        .map(|d| PciStandardHeaderDevice::new(d.clone()).unwrap())
        .map(|d| Xhci::try_from(d).unwrap())
        .next()
        .unwrap();
    serial_println!("capabilities: {:#?}", xhci.capabilities.read());
    serial_println!("usb status: {:#?}", xhci.operational.read());

    change_thread_priority(Priority::Low);

    panic!("kernel_main returned");
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
