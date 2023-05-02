use crate::{map_page, serial_println};
use acpi::{AcpiHandler, AcpiTables, InterruptModel, PhysicalMapping, PlatformInfo};
use alloc::rc::Rc;
use bootloader_api::BootInfo;
use conquer_once::spin::OnceCell;
use core::assert_matches::assert_matches;
use core::ptr::NonNull;
use spin::Mutex;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub static INTERRUPT_MODEL: OnceCell<InterruptModel> = OnceCell::uninit();

pub fn init(boot_info: &'static BootInfo) {
    if boot_info.rsdp_addr.into_option().is_none() {
        serial_println!("no rsdp found");
        return;
    }
    let rsdp = boot_info.rsdp_addr.into_option().unwrap();

    let result = unsafe { AcpiTables::from_rsdp(KernelAcpi::new(), rsdp as usize) };
    if let Err(e) = result {
        serial_println!("acpi error: {:#?}", e);
        return;
    }
    let tables = result.unwrap();
    if let Ok(platform_info) = PlatformInfo::new(&tables) {
        assert_matches!(platform_info.interrupt_model, InterruptModel::Apic(_)); // TODO: remove and support the other one(s), too
        INTERRUPT_MODEL.init_once(|| platform_info.interrupt_model);
    }
}

#[derive(Clone, Debug)]
pub struct KernelAcpi {
    addr: Rc<Mutex<u64>>,
}

impl KernelAcpi {
    pub fn new() -> Self {
        KernelAcpi {
            addr: Rc::new(Mutex::new(0x1111_1122_0000)), // TODO: make dynamic
        }
    }
}

impl AcpiHandler for KernelAcpi {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(*self.addr.lock()));
        assert!(size < Size4KiB::SIZE as usize);
        *self.addr.lock() += Size4KiB::SIZE;

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
