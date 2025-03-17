use x86_64::registers::model_specific::GsBase;

pub struct ExecutionContext {
    cpu_id: usize,
    lapid_id: usize,
}

impl From<&limine::mp::Cpu> for ExecutionContext {
    fn from(cpu: &limine::mp::Cpu) -> Self {
        ExecutionContext {
            cpu_id: cpu.id as usize,
            lapid_id: cpu.extra as usize,
        }
    }
}

impl ExecutionContext {
    pub fn try_load() -> Option<&'static Self> {
        let ctx = GsBase::read();
        if ctx.is_null() {
            None
        } else {
            Some(unsafe { &*ctx.as_ptr() })
        }
    }

    pub fn load() -> &'static Self {
        Self::try_load().expect("could not load cpu context")
    }

    pub fn cpu_id(&self) -> usize {
        self.cpu_id
    }

    pub fn lapic_id(&self) -> usize {
        self.lapid_id
    }
}
