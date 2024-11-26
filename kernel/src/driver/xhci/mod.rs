use crate::driver::xhci::error::XhciError;
use crate::mem::virt::OwnedInterval;
use crate::unmap_page;
use core::fmt::Debug;
use core::num::NonZeroU8;
use core::ops::Deref;
use volatile::VolatilePtr;
use x86_64::structures::paging::{Page, PageSize, Size4KiB};

use crate::driver::pci::PciDevice;
use crate::driver::xhci::extended::ExtendedCapabilities;
pub use capabilities::*;
pub use operational::*;
pub use portpmsc::*;
pub use portsc::*;
pub use psi::*;
pub use registers::*;
pub use supported_protocol_capability::*;

mod capabilities;
mod error;
mod extended;
mod operational;
mod portpmsc;
mod portsc;
mod psi;
mod registers;
mod supported_protocol_capability;

#[derive(Debug)]
pub struct XhciRegisters<'a> {
    interval: OwnedInterval<'a>,
    registers: Registers<'a>,
}

impl<'a> Deref for XhciRegisters<'a> {
    type Target = Registers<'a>;

    fn deref(&self) -> &Self::Target {
        &self.registers
    }
}

impl TryFrom<PciDevice> for XhciRegisters<'_> {
    type Error = XhciError;

    fn try_from(_pci_device: PciDevice) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Drop for XhciRegisters<'_> {
    fn drop(&mut self) {
        let start_addr = self.interval.start();
        (start_addr..(start_addr + (self.interval.size() - 1)))
            .step_by(Size4KiB::SIZE as usize)
            .map(Page::<Size4KiB>::containing_address)
            .for_each(|page| {
                unmap_page!(page, Size4KiB);
            });
    }
}

impl XhciRegisters<'_> {
    pub fn portsc(&self, port: NonZeroU8) -> VolatilePtr<'_, PortSc> {
        let addr = unsafe {
            self.operational
                .as_raw_ptr()
                .cast::<u8>()
                .add(0x400)
                .add(0x10 * (port.get() - 1) as usize)
                .cast()
        };
        unsafe { VolatilePtr::new(addr) }
    }

    pub fn portpmsc<T: PortPmsc>(&self, port: NonZeroU8) -> VolatilePtr<'_, T> {
        let addr = unsafe {
            self.operational
                .as_raw_ptr()
                .cast::<u8>()
                .add(0x404)
                .add(0x10 * (port.get() - 1) as usize)
                .cast()
        };
        unsafe { VolatilePtr::new(addr) }
    }

    pub fn extended_capabilities(&self) -> ExtendedCapabilitiesIter<'_> {
        ExtendedCapabilitiesIter {
            xhci: self,
            next: None,
            fused_finished: false,
        }
    }
}

pub struct ExtendedCapabilitiesIter<'a> {
    xhci: &'a XhciRegisters<'a>,
    next: Option<VolatilePtr<'a, ExtendedCapabilities>>,
    fused_finished: bool,
}

impl<'a> Iterator for ExtendedCapabilitiesIter<'a> {
    type Item = VolatilePtr<'a, ExtendedCapabilities>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fused_finished {
            return None;
        }

        let next = unsafe {
            VolatilePtr::new(if let Some(next) = self.next {
                let next_offset = next.read().next_raw();
                if next_offset == 0 {
                    self.fused_finished = true;
                    return None;
                }
                next.as_raw_ptr()
                    .cast::<u8>()
                    .add((next_offset as usize) << 2)
                    .cast()
            } else {
                self.xhci
                    .capabilities
                    .as_raw_ptr()
                    .cast::<u8>()
                    .add((self.xhci.capabilities.read().hccparams1.xecp() as usize) << 2)
                    .cast()
            })
        };
        self.next = Some(next);
        self.next
    }
}
