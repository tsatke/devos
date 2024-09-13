use crate::driver::xhci::{Capabilities, CapabilitiesVolatileFieldAccess, Operational};

use core::fmt::Debug;
use core::ptr::NonNull;
use volatile::access::ReadWrite;
use volatile::{VolatileFieldAccess, VolatilePtr};
use x86_64::VirtAddr;

#[repr(C)]
#[derive(Debug)]
pub struct Registers<'a> {
    pub capabilities: VolatilePtr<'a, Capabilities>,
    pub operational: VolatilePtr<'a, Operational>,
    pub port: VolatilePtr<'a, Port>,
    pub runtime: VolatilePtr<'a, Runtime>,
}

impl Registers<'_> {
    pub fn new(base: VirtAddr) -> Self {
        let capabilities = unsafe { VolatilePtr::new(NonNull::new(base.as_mut_ptr::<Capabilities>()).unwrap()) };

        let caplength = capabilities.caplength().read();
        let operational_base = base + caplength as u64;
        assert!(base + size_of::<Capabilities>() < operational_base, "capabilities registers should not overlap into operational registers");
        let operational = unsafe { VolatilePtr::new(NonNull::new(operational_base.as_mut_ptr::<Operational>()).unwrap()) };

        let port_base = operational_base + 0x400_usize;
        assert!(operational_base + size_of::<Operational>() < port_base, "operational registers should not overlap into port registers");
        let port = unsafe { VolatilePtr::new(NonNull::new(port_base.as_mut_ptr::<Port>()).unwrap()) };

        let rtsoff = capabilities.rtsoff().read();
        let runtime_base = base + rtsoff as u64;
        let runtime = unsafe { VolatilePtr::new(NonNull::new(runtime_base.as_mut_ptr::<Runtime>()).unwrap()) };

        Self { capabilities, operational, port, runtime }
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Port {
    // TODO: implement
}

#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Runtime {
    #[access(ReadWrite)]
    mfindex: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Interrupter {
    #[access(ReadWrite)]
    iman: u32,
    #[access(ReadWrite)]
    imod: u32,
    #[access(ReadWrite)]
    erstsz: u64,
    #[access(ReadWrite)]
    erstba: u64,
    #[access(ReadWrite)]
    erdp: u64,
}

