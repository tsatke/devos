use crate::mcore::mtask::scheduler::Scheduler;
use crate::U64Ext;
use core::cell::UnsafeCell;
use x86_64::registers::model_specific::KernelGsBase;

#[derive(Debug)]
pub struct ExecutionContext {
    cpu_id: usize,
    lapid_id: usize,

    scheduler: UnsafeCell<Scheduler>,
}

impl From<&limine::mp::Cpu> for ExecutionContext {
    fn from(cpu: &limine::mp::Cpu) -> Self {
        ExecutionContext {
            cpu_id: cpu.id as usize,
            lapid_id: cpu.extra.into_usize(),
            scheduler: UnsafeCell::new(Scheduler::new_cpu_local()),
        }
    }
}

impl ExecutionContext {
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
    pub unsafe fn scheduler(&self) -> &mut Scheduler {
        unsafe { &mut *self.scheduler.get() }
    }
}
