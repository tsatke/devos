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
use crate::mem::virt::Interval;
use crate::mem::Size;
use crate::process::vmm;
use crate::Result;

pub static INTERRUPT_MODEL: OnceCell<InterruptModel<Global>> = OnceCell::uninit();

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
    addr: Rc<Mutex<u64>>,
    reserved_memory_interval: Interval,
}

impl KernelAcpi {
    pub fn new() -> Self {
        let interval = vmm()
            .reserve(Size::MiB(1).bytes())
            .expect("failed to reserve memory for acpi");
        KernelAcpi {
            addr: Rc::new(Mutex::new(interval.start().as_u64())),
            reserved_memory_interval: interval,
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
            let mut guard = self.addr.lock();
            if *guard
                > (self.reserved_memory_interval.start() + self.reserved_memory_interval.size())
                    .as_u64()
            {
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
