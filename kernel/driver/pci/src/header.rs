use crate::raw::read_config_double_word;
use crate::{Error, PciDevice};
use core::ops::Deref;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PciHeaderType {
    Standard = 0x00,
    Pci2PciBridge = 0x01,
    CardBusBridge = 0x02,
}

impl TryFrom<u8> for PciHeaderType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Standard,
            0x01 => Self::Pci2PciBridge,
            0x02 => Self::CardBusBridge,
            _ => return Err(Error::UnknownHeaderType(value)),
        })
    }
}

pub struct PciStandardHeaderDevice {
    inner: PciDevice,
}

impl Deref for PciStandardHeaderDevice {
    type Target = PciDevice;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PciStandardHeaderDevice {
    const OFFSET_BAR0: u8 = 0x10;
    const OFFSET_BAR1: u8 = 0x14;
    const OFFSET_BAR2: u8 = 0x18;
    const OFFSET_BAR3: u8 = 0x1C;
    const OFFSET_BAR4: u8 = 0x20;
    const OFFSET_BAR5: u8 = 0x24;

    pub fn new(inner: PciDevice) -> Result<Self, Error> {
        let header_type = inner.header_type();
        if header_type != PciHeaderType::Standard {
            return Err(Error::NotStandardHeader(header_type));
        }
        Ok(PciStandardHeaderDevice { inner })
    }

    pub fn bar0(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR0)
    }

    pub fn bar1(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR1)
    }

    pub fn bar2(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR2)
    }

    pub fn bar3(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR3)
    }

    pub fn bar4(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR4)
    }

    pub fn bar5(&self) -> u32 {
        self.read_bar(Self::OFFSET_BAR5)
    }

    fn read_bar(&self, bar_offset: u8) -> u32 {
        unsafe {
            read_config_double_word(
                self.inner.bus(),
                self.inner.slot(),
                self.inner.function(),
                bar_offset,
            )
        }
    }
}

pub struct Pci2PciBridge {
    inner: PciDevice,
}

impl Deref for Pci2PciBridge {
    type Target = PciDevice;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Pci2PciBridge {
    pub fn new(inner: PciDevice) -> Result<Self, Error> {
        let header_type = inner.header_type();
        if header_type != PciHeaderType::Pci2PciBridge {
            return Err(Error::NotPCI2PCIBridge(header_type));
        }
        Ok(Pci2PciBridge { inner })
    }
}
