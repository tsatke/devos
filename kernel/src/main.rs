#![no_std]
#![no_main]

extern crate alloc;

use core::num::NonZeroUsize;
use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use kernel::arch::panic::handle_panic;
use kernel::mem::virt::Interval;
use kernel::process::{change_thread_priority, vmm, Priority, Process};
use kernel::{bootloader_config, kernel_init, map_page, process, serial_println};
use pci::{BaseAddressRegister, PciDeviceClass, PciStandardHeaderDevice, SerialBusSubClass};
use x86_64::structures::paging::frame::PhysFrameRangeInclusive;
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use xhci::accessor::Mapper;

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

    pci::devices()
        .filter_map(|device| {
            if !matches!(device.class(), PciDeviceClass::SerialBusController(SerialBusSubClass::USBController)) {
                return None;
            }
            let shd = PciStandardHeaderDevice::new(device.clone()).ok()?;
            let bar0 = shd.bar0();
            let (addr, size) = match bar0 {
                BaseAddressRegister::MemorySpace64(bar) => (bar.addr, bar.size),
                _ => return None,
            };

            let registers = unsafe { xhci::Registers::new(addr as usize, XhciMapper) };

            Some(shd.bar0())
        })
        .for_each(|d| serial_println!("{:#x?}", d));

    change_thread_priority(Priority::Low);
    panic!("kernel_main returned");
}

#[derive(Debug, Copy, Clone)]
struct XhciMapper;

impl Mapper for XhciMapper {
    unsafe fn map(&mut self, phys_start: usize, bytes: usize) -> NonZeroUsize {
        let frames = {
            let start = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(phys_start as u64));
            let end = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(phys_start as u64 + bytes as u64 - 1));
            PhysFrameRangeInclusive { start, end }
        };

        let interval = vmm().reserve(bytes).unwrap().leak();
        serial_println!("allocated interval at {:#p} size {:#x}", interval.start(), interval.size());

        for (i, frame) in frames.enumerate() {
            let vaddr = interval.start() + (i as u64 * frame.size());
            map_page!(
                Page::containing_address(vaddr),
                frame,
                Size4KiB,
                PageTableFlags::PRESENT
            );
        }

        NonZeroUsize::new(interval.start().as_u64() as usize).unwrap()
    }

    fn unmap(&mut self, virt_start: usize, bytes: usize) {
        serial_println!("unmap: {:#p} size {:#x}", VirtAddr::new(virt_start as u64), bytes);
        assert!(vmm().release(Interval::new(VirtAddr::new(virt_start as u64), bytes)));
    }
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
