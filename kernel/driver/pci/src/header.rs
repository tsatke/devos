use core::ops::Deref;

use crate::raw::{read_config_double_word, write_config_double_word};
use crate::{Error, PciDevice};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BaseAddressRegister {
    MemorySpace32(MemorySpace32Bar),
    MemorySpace64(MemorySpace64Bar),
    IoSpace(u32),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MemorySpace32Bar {
    pub addr: u32,
    pub prefetchable: bool,
    pub size: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MemorySpace64Bar {
    pub addr: u64,
    pub prefetchable: bool,
    pub size: usize,
}

impl BaseAddressRegister {
    pub fn memory_space_32(&self) -> Option<MemorySpace32Bar> {
        match self {
            Self::MemorySpace32(bar) => Some(*bar),
            _ => None,
        }
    }

    pub fn memory_space_64(&self) -> Option<MemorySpace64Bar> {
        match self {
            Self::MemorySpace64(bar) => Some(*bar),
            _ => None,
        }
    }

    pub fn io_space(&self) -> Option<u32> {
        match self {
            Self::IoSpace(bar) => Some(*bar),
            _ => None,
        }
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

    pub fn bar0(&self) -> BaseAddressRegister {
        self.decode_bar(Self::OFFSET_BAR0, Some(Self::OFFSET_BAR1))
    }

    pub fn bar1(&self) -> BaseAddressRegister {
        self.decode_bar(Self::OFFSET_BAR1, Some(Self::OFFSET_BAR2))
    }

    pub fn bar2(&self) -> BaseAddressRegister {
        self.decode_bar(Self::OFFSET_BAR2, Some(Self::OFFSET_BAR3))
    }

    pub fn bar3(&self) -> BaseAddressRegister {
        self.decode_bar(Self::OFFSET_BAR3, Some(Self::OFFSET_BAR4))
    }

    pub fn bar4(&self) -> BaseAddressRegister {
        self.decode_bar(Self::OFFSET_BAR4, Some(Self::OFFSET_BAR5))
    }

    pub fn bar5(&self) -> BaseAddressRegister {
        self.decode_bar(Self::OFFSET_BAR5, None)
    }

    fn decode_bar(&self, bar_offset: u8, next_bar_offset: Option<u8>) -> BaseAddressRegister {
        let bar = self.read_bar_raw(bar_offset);
        let next_bar = next_bar_offset.map(|offset| self.read_bar_raw(offset));

        if bar & 0x01 > 0 {
            // io space bar
            BaseAddressRegister::IoSpace(bar & !0b11)
        } else {
            // memory space bar
            let prefetchable = bar & 0b100 > 0;

            // FIXME: disable and restore I/O and memory decode bit

            self.write_bar_raw(bar_offset, !0);
            let new_bar = self.read_bar_raw(bar_offset);
            self.write_bar_raw(bar_offset, bar); // restore the original value

            let size = (!(new_bar & !0b1111) + 1) as usize;

            if (bar >> 1) & 0x02 > 0 {
                // 64bit bar
                let addr = (bar & !0b1111) as u64 | ((next_bar.unwrap() as u64) << 32);
                BaseAddressRegister::MemorySpace64(MemorySpace64Bar {
                    addr,
                    prefetchable,
                    size,
                })
            } else {
                let addr = bar & !0b1111;
                BaseAddressRegister::MemorySpace32(MemorySpace32Bar {
                    addr,
                    prefetchable,
                    size,
                })
            }
        }
    }

    pub fn bar0_raw(&self) -> u32 {
        self.read_bar_raw(Self::OFFSET_BAR0)
    }

    pub fn bar1_raw(&self) -> u32 {
        self.read_bar_raw(Self::OFFSET_BAR1)
    }

    pub fn bar2_raw(&self) -> u32 {
        self.read_bar_raw(Self::OFFSET_BAR2)
    }

    pub fn bar3_raw(&self) -> u32 {
        self.read_bar_raw(Self::OFFSET_BAR3)
    }

    pub fn bar4_raw(&self) -> u32 {
        self.read_bar_raw(Self::OFFSET_BAR4)
    }

    pub fn bar5_raw(&self) -> u32 {
        self.read_bar_raw(Self::OFFSET_BAR5)
    }

    fn read_bar_raw(&self, bar_offset: u8) -> u32 {
        unsafe {
            read_config_double_word(
                self.inner.bus(),
                self.inner.slot(),
                self.inner.function(),
                bar_offset,
            )
        }
    }

    fn write_bar_raw(&self, bar_offset: u8, value: u32) {
        unsafe {
            write_config_double_word(
                self.inner.bus(),
                self.inner.slot(),
                self.inner.function(),
                bar_offset,
                value,
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
