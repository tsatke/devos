use crate::mcore::mtask::scheduler::Scheduler;
use crate::U64Ext;
use core::cell::UnsafeCell;
use x86_64::registers::model_specific::KernelGsBase;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::idt::InterruptDescriptorTable;

#[derive(Debug)]
pub struct ExecutionContext {
    cpu_id: usize,
    lapid_id: usize,

    _gdt: &'static GlobalDescriptorTable,
    _idt: &'static InterruptDescriptorTable,

    scheduler: UnsafeCell<Scheduler>,
}

impl ExecutionContext {
    pub fn new(
        cpu: &limine::mp::Cpu,
        gdt: &'static GlobalDescriptorTable,
        idt: &'static InterruptDescriptorTable,
    ) -> Self {
        ExecutionContext {
            cpu_id: cpu.id as usize,
            lapid_id: cpu.extra.into_usize(),
            _gdt: gdt,
            _idt: idt,
            scheduler: UnsafeCell::new(Scheduler::new_cpu_local()),
        }
    }

    #[must_use]
    pub fn try_load() -> Option<&'static Self> {
        let ctx = KernelGsBase::read();
        if ctx.is_null() {
            None
        } else {
            Some(unsafe { &*ctx.as_ptr() })
        }
    }

    /// # Panics
    /// This function panics if the execution context could not be loaded.
    /// This could happen if no execution context exists yet, or the pointer
    /// or its memory in `KernelGSBase` is invalid.
    #[must_use]
    pub fn load() -> &'static Self {
        Self::try_load().expect("could not load cpu context")
    }

    #[must_use]
    pub fn cpu_id(&self) -> usize {
        self.cpu_id
    }

    #[must_use]
    pub fn lapic_id(&self) -> usize {
        self.lapid_id
    }

    /// Creates and returns a mutable reference to the scheduler.
    ///
    /// # Safety
    /// The caller must ensure that only one mutable reference
    /// to the scheduler exists at any time.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn scheduler(&self) -> &mut Scheduler {
        unsafe { &mut *self.scheduler.get() }
    }
}
