#![no_std]
#![no_main]

use core::slice::from_raw_parts_mut;
use kernel::driver::vga::{vga_devices, VgaDevice};
use kernel::limine::BASE_REVISION;
use kernel::mem::address_space::AddressSpace;
use kernel::mem::virt::VirtualMemoryHigherHalf;
use kernel::{mcore, U64Ext};
use log::error;
use x86_64::instructions::hlt;
use x86_64::structures::paging::{PageSize, PageTableFlags, Size4KiB};

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    if let Some(&vga_phys_mem) = vga_devices()
        .lock()
        .iter()
        .next()
        .map(VgaDevice::physical_memory)
    {
        let size = vga_phys_mem.size();
        let pages = size.div_ceil(Size4KiB::SIZE);
        let segment = VirtualMemoryHigherHalf::reserve(pages.into_usize())
            .expect("should have enough memory for framebuffer");
        AddressSpace::kernel()
            .map_range(
                &*segment,
                vga_phys_mem,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            )
            .unwrap();
        let slice = unsafe {
            from_raw_parts_mut(segment.start.as_mut_ptr::<u8>(), segment.len.into_usize())
        };
        slice.fill(0xCE);
    }

    mcore::turn_idle()
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    let location = info.location().unwrap();
    error!(
        "kernel panicked at {}:{}:{}:\n{}",
        location.file(),
        location.line(),
        location.column(),
        info.message(),
    );
    loop {
        hlt();
    }
}
