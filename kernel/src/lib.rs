#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(array_chunks)]
#![feature(assert_matches)]
#![feature(const_mut_refs)]
#![feature(error_in_core)]
#![feature(exclusive_range_pattern)]
#![feature(iter_array_chunks)]
#![feature(naked_functions)]
#![feature(negative_impls)]
#![feature(never_type)]

extern crate alloc;

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig};
use conquer_once::spin::OnceCell;
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;

pub use error::Result;

use crate::arch::{gdt, idt};
use crate::io::vfs;
use crate::mem::Size;

pub mod acpi;
pub mod apic;
pub mod arch;
mod error;
pub mod io;
pub mod mem;
pub mod process;
pub mod qemu;
pub mod screen;
pub mod syscall;
pub mod timer;

const KERNEL_STACK_SIZE: Size = Size::KiB(128);

pub static KERNEL_CODE_ADDR: OnceCell<VirtAddr> = OnceCell::uninit();
pub static KERNEL_CODE_LEN: OnceCell<usize> = OnceCell::uninit();

pub const fn bootloader_config() -> BootloaderConfig {
    let mut config = BootloaderConfig::new_default();
    config.mappings.page_table_recursive = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::Dynamic; // TODO: we don't need a frame buffer at all, since we get the frame buffer memory from the pci device tree
    config.kernel_stack_size = KERNEL_STACK_SIZE.bytes() as u64;
    config
}

pub fn kernel_init(boot_info: &'static BootInfo) -> Result<()> {
    gdt::init();
    idt::init();
    mem::init(boot_info)?;
    acpi::init(boot_info)?;
    apic::init()?;
    vfs::init();

    KERNEL_CODE_ADDR.init_once(|| VirtAddr::new(boot_info.kernel_image_offset));
    KERNEL_CODE_LEN.init_once(|| boot_info.kernel_len as usize);

    interrupts::enable();

    screen::init();

    Ok(())
}

#[cfg(feature = "kernel_test")]
mod tests {
    use kernel_test_framework::kernel_test;

    #[kernel_test]
    #[allow(clippy::eq_op)]
    fn test_it_works() {
        assert_eq!(2 + 2, 4);
    }
}
