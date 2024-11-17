use crate::driver::acpi::acpi_tables;
use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::process::vmm;
use acpi::HpetInfo;
use alloc::string::ToString;
use bitfield::bitfield;
use conquer_once::spin::OnceCell;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use spin::Mutex;
use volatile::access::NoAccess;
use volatile::access::ReadOnly;
use volatile::access::ReadWrite;
use volatile::{VolatileFieldAccess, VolatilePtr};
use x86_64::structures::paging::{PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

static HPET: OnceCell<Mutex<VolatileHpetPtr>> = OnceCell::uninit();

pub fn hpet() -> Option<&'static Mutex<VolatileHpetPtr<'static>>> {
    HPET.get()
}

pub fn init() {
    let acpi_tables = acpi_tables().unwrap();
    let guard = acpi_tables.lock();

    let hpet_info = HpetInfo::new(&guard).unwrap();
    let base_address = PhysAddr::try_new(hpet_info.base_address as u64).unwrap();

    let addr = vmm()
        .allocate_memory_backed_vmobject(
            "hpet".to_string(),
            MapAt::Anywhere,
            Size4KiB::SIZE as usize,
            AllocationStrategy::MapNow(&[PhysFrame::containing_address(base_address)]),
            PageTableFlags::PRESENT
                | PageTableFlags::NO_CACHE
                | PageTableFlags::NO_EXECUTE
                | PageTableFlags::WRITABLE,
        )
        .unwrap();

    let hpet_volatile_ptr = unsafe { VolatilePtr::new(NonNull::new(addr.as_mut_ptr()).unwrap()) };
    HPET.init_once(|| {
        Mutex::new(VolatileHpetPtr {
            ptr: hpet_volatile_ptr,
        })
    });
}

pub struct VolatileHpetPtr<'a> {
    ptr: VolatilePtr<'a, Hpet>,
}

unsafe impl Send for VolatileHpetPtr<'_> {}

impl<'a> Deref for VolatileHpetPtr<'a> {
    type Target = VolatilePtr<'a, Hpet>;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl<'a> DerefMut for VolatileHpetPtr<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ptr
    }
}

#[repr(C)]
#[derive(Debug, VolatileFieldAccess)]
pub struct Hpet {
    #[access(ReadOnly)]
    pub capabilities_and_id: CapabilitiesAndId,
    #[access(NoAccess)]
    _pad1: MaybeUninit<u64>,
    #[access(ReadWrite)]
    pub config: Config,
    #[access(NoAccess)]
    _pad2: MaybeUninit<u64>,
    #[access(ReadWrite)]
    pub interrupt_status: u64,
    #[access(NoAccess)]
    _pad3: MaybeUninit<[u64; 19]>,
    pub main_counter_value: u64,
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct CapabilitiesAndId(u64);
    impl Debug;

    pub u32, counter_clk_period, _: 63, 32;
    pub u16, vendor_id, _: 31, 16;
    pub bool, legacy_replacement, _: 15;
    pub bool, count_size_cap, _: 13;
    pub u8, num_timers, _: 12, 8;
    pub u8, rev_id, _: 7, 0;
}

bitfield! {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct Config(u64);
    impl Debug;

    pub bool, legacy_replacement_cnf, set_legacy_replacement_cnf: 1;
    pub bool, enable_cnf, set_enable_cnf: 0;
}
