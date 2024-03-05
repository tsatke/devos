use alloc::alloc::Global;
use alloc::format;
use alloc::rc::Rc;
use core::assert_matches::assert_matches;
use core::ptr::NonNull;

use acpi::{AcpiHandler, AcpiTables, InterruptModel, PhysicalMapping, PlatformInfo};
use bootloader_api::BootInfo;
use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use crate::map_page;
use crate::mem::Size;
use crate::Result;

pub static INTERRUPT_MODEL: OnceCell<InterruptModel<Global>> = OnceCell::uninit();

pub static KERNEL_ACPI_ADDR: OnceCell<VirtAddr> = OnceCell::uninit();
pub static KERNEL_ACPI_LEN: Size = Size::MiB(1);

pub fn init(boot_info: &'static BootInfo) -> Result<()> {
    let rsdp = boot_info.rsdp_addr.into_option().ok_or("no rsdp found")?;

    let result = unsafe { AcpiTables::from_rsdp(KernelAcpi::new(), rsdp as usize) };
    if let Err(e) = result {
        panic!("acpi error: {:#?}", e); // FIXME: this currently occurs while booting in BIOS mode rather than UEFI
    }
    let tables = result.map_err(|e| format!("acpi error: {:#?}", e))?;
    if let Ok(platform_info) = PlatformInfo::new(&tables) {
        assert_matches!(platform_info.interrupt_model, InterruptModel::Apic(_)); // TODO: remove and support the other one(s), too
        INTERRUPT_MODEL.init_once(|| platform_info.interrupt_model);
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct KernelAcpi {
    start_addr: Rc<Mutex<u64>>,
    end_addr: u64,
}

impl KernelAcpi {
    pub fn new() -> Self {
        let start_addr = KERNEL_ACPI_ADDR
            .get()
            .expect("kernel acpi address not initialized")
            .as_u64();
        let end_addr = start_addr + KERNEL_ACPI_LEN.bytes() as u64;
        KernelAcpi {
            start_addr: Rc::new(Mutex::new(start_addr)),
            end_addr,
        }
    }
}

impl !Default for KernelAcpi {}

impl AcpiHandler for KernelAcpi {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let page = {
            let mut guard = self.start_addr.lock();
            if *guard + Page::<Size4KiB>::SIZE > self.end_addr {
                panic!("acpi memory exhausted");
            }

            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(*guard));
            assert!(size < Size4KiB::SIZE as usize);
            *guard += Size4KiB::SIZE;
            page
        };

        map_page!(
            page,
            PhysFrame::containing_address(PhysAddr::new(physical_address as u64)),
            Size4KiB,
            PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::NO_CACHE
                | PageTableFlags::WRITE_THROUGH
        );
        PhysicalMapping::new(
            physical_address,
            NonNull::new(page.start_address().as_mut_ptr()).unwrap(),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        let _ = region;
        // FIXME: don't let the phys page leak
    }
}
