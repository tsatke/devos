#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(assert_matches)]
#![feature(box_into_inner)]
#![feature(extend_one_unchecked)]
#![feature(iter_array_chunks)]
#![feature(naked_functions)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(try_with_capacity)]
#![feature(vec_push_within_capacity)]
extern crate alloc;

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig};
use conquer_once::spin::OnceCell;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{Page, Size4KiB};
use x86_64::VirtAddr;

pub use error::Result;

use crate::acpi::{KERNEL_ACPI_ADDR, KERNEL_ACPI_LEN};
use crate::apic::{KERNEL_APIC_ADDR, KERNEL_APIC_LEN};
use crate::arch::{gdt, idt};
use crate::io::vfs;
use crate::mem::virt::heap::{KERNEL_HEAP_ADDR, KERNEL_HEAP_LEN};
use crate::mem::Size;

pub mod acpi;
pub mod apic;
pub mod arch;
pub mod driver;
mod error;
pub mod foundation;
pub mod io;
pub mod mem;
pub mod process;
pub mod qemu;
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
    config.mappings.dynamic_range_start = Some(0xffff_8000_0000_0000);
    config.mappings.dynamic_range_end = Some(0xffff_ffff_ffff_ffff);
    config
}

pub fn kernel_init(boot_info: &'static BootInfo) -> Result<()> {
    // TODO: probably map guard pages in between the different sections, just in case
    let kernel_code_addr = VirtAddr::new(boot_info.kernel_image_offset);
    let kernel_code_len = boot_info.kernel_len as usize;
    let kernel_heap_addr = (kernel_code_addr + kernel_code_len).align_up(Page::<Size4KiB>::SIZE);
    let kernel_heap_len = KERNEL_HEAP_LEN.bytes();
    let kernel_acpi_addr = (kernel_heap_addr + kernel_heap_len).align_up(Page::<Size4KiB>::SIZE);
    let kernel_acpi_len = KERNEL_ACPI_LEN.bytes();
    let kernel_apic_addr = (kernel_acpi_addr + kernel_acpi_len).align_up(Page::<Size4KiB>::SIZE);
    let _kernel_apic_len = KERNEL_APIC_LEN.bytes();

    KERNEL_CODE_ADDR.init_once(|| kernel_code_addr);
    KERNEL_CODE_LEN.init_once(|| kernel_code_len);
    KERNEL_HEAP_ADDR.init_once(|| kernel_heap_addr);
    KERNEL_ACPI_ADDR.init_once(|| kernel_acpi_addr);
    KERNEL_APIC_ADDR.init_once(|| kernel_apic_addr);

    gdt::init();
    idt::init();
    syscall::init();
    mem::init(boot_info)?; // sets up address space, thus implies process::init and scheduler::init
    acpi::init(boot_info)?;
    apic::init()?;
    vfs::init();

    interrupts::enable();

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
