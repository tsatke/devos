use crate::driver::pci::{BaseAddressRegister, PciDeviceClass, PciStandardHeaderDevice, SerialBusSubClass};
use crate::driver::xhci::error::XhciError;
use crate::mem::virt::OwnedInterval;
use crate::process::vmm;
use crate::{map_page, unmap_page};
use core::ops::Deref;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub use error::*;
pub use registers::*;

mod error;
mod registers;

#[derive(Debug)]
pub struct Xhci<'a> {
    interval: OwnedInterval<'a>,
    registers: Registers<'a>,
}

impl<'a> Deref for Xhci<'a> {
    type Target = Registers<'a>;

    fn deref(&self) -> &Self::Target {
        &self.registers
    }
}

impl TryFrom<PciStandardHeaderDevice> for Xhci<'_> {
    type Error = XhciError;

    fn try_from(pci_device: PciStandardHeaderDevice) -> Result<Self, Self::Error> {
        if !(matches!(pci_device.class(), PciDeviceClass::SerialBusController(SerialBusSubClass::USBController)) && pci_device.prog_if() == 0x30) {
            return Err(XhciError::NotUsb);
        }

        let (phys_addr, size) = match pci_device.bar0() {
            BaseAddressRegister::MemorySpace64(bar) => (bar.addr, bar.size),
            _ => return Err(XhciError::NotUsb),
        };
        let frames = {
            let start = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(phys_addr));
            let end = start + (size - 1) as u64;
            PhysFrame::<Size4KiB>::range_inclusive(start, end)
        };

        let interval = vmm().reserve(size).map_err(|vmm_err| XhciError::VmmError(vmm_err))?;
        debug_assert_eq!(size, interval.size());
        let start_addr = interval.start();
        (start_addr..(start_addr + (size - 1))).step_by(Size4KiB::SIZE as usize)
            .map(Page::<Size4KiB>::containing_address)
            .zip(frames)
            .for_each(|(page, frame)| {
                map_page!(
                    page,
                    frame,
                    Size4KiB,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE | PageTableFlags::NO_CACHE
                );
            });

        let registers = Registers::new(start_addr);

        Ok(Self {
            interval,
            registers,
        })
    }
}

impl Drop for Xhci<'_> {
    fn drop(&mut self) {
        let start_addr = self.interval.start();
        (start_addr..(start_addr + (self.interval.size() - 1))).step_by(Size4KiB::SIZE as usize)
            .map(Page::<Size4KiB>::containing_address)
            .for_each(|page| {
                unmap_page!(
                    page,
                    Size4KiB
                );
            });
    }
}

impl Xhci<'_> {}

