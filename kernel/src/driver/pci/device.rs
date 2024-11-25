use crate::driver::pci::raw::{
    read_config_double_word, read_config_half_word, read_config_word, write_config_double_word,
    OFFSET_BIST, OFFSET_CLASS_SUBCLASS, OFFSET_HEADER_TYPE, OFFSET_INTERRUPT_LINE,
    OFFSET_INTERRUPT_PIN, OFFSET_PROG_IF_REVISION_ID, OFFSET_STATUS,
};
use crate::driver::pci::{InterruptPin, PciDeviceClass, PciError, PciHeaderType, Status, BIST};
use core::fmt::Formatter;
use derive_more::Display;

#[derive(Debug)]
pub struct PciDevice {
    bus: u8,
    slot: u8,
    function: u8,
    vendor: u16,
    device: u16,
    header_type: PciHeaderType,
    multi_function: bool,
    class: PciDeviceClass,
    interrupt_pin: InterruptPin,
}

impl Display for PciDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "vendor=0x{:04x} device=0x{:04x}, slot: {:02x}:{:02x}.{:x}",
            self.vendor(),
            self.device(),
            self.bus(),
            self.slot(),
            self.function(),
        )
    }
}

impl PciDevice {
    /// Create a new pci device from the given parameters.
    ///
    /// # Safety
    ///
    /// Creating a new pci device is unsafe because this
    /// reads from memory, which could have unintended
    /// effects. Also, the caller has to ensure that this
    /// is only called once for every combination of parameters.
    pub unsafe fn new(
        bus: u8,
        slot: u8,
        function: u8,
        vendor: u16,
        device: u16,
    ) -> Result<Self, PciError> {
        let header_type_raw = read_config_half_word(bus, slot, function, OFFSET_HEADER_TYPE);
        let header_type = PciHeaderType::try_from(header_type_raw & ((1 << 7) - 1))?;
        let multi_function = header_type_raw & (1 << 7) > 0;
        let class =
            PciDeviceClass::try_from(read_config_word(bus, slot, function, OFFSET_CLASS_SUBCLASS))?;
        let interrupt_pin = InterruptPin::try_from(read_config_half_word(
            bus,
            slot,
            function,
            OFFSET_INTERRUPT_PIN,
        ))?;
        Ok(PciDevice {
            bus,
            slot,
            function,
            vendor,
            device,
            header_type,
            multi_function,
            class,
            interrupt_pin,
        })
    }

    pub fn class(&self) -> PciDeviceClass {
        self.class
    }

    pub fn prog_if(&self) -> u8 {
        let guard = self;
        unsafe {
            (read_config_word(
                guard.bus,
                guard.slot,
                guard.function,
                OFFSET_PROG_IF_REVISION_ID,
            ) >> 8) as u8
        }
    }

    pub fn revision_id(&self) -> u8 {
        let guard = self;
        unsafe {
            read_config_word(
                guard.bus,
                guard.slot,
                guard.function,
                OFFSET_PROG_IF_REVISION_ID,
            ) as u8
        }
    }

    pub fn status(&self) -> Status {
        let guard = self;
        let status =
            unsafe { read_config_word(guard.bus, guard.slot, guard.function, OFFSET_STATUS) };
        Status::from_bits_truncate(status)
    }

    pub fn bist(&self) -> BIST {
        let guard = self;
        let bist = unsafe { read_config_word(guard.bus, guard.slot, guard.function, OFFSET_BIST) };
        BIST::from_bits_truncate(bist as u8)
    }

    pub fn interrupt_line(&self) -> Option<u8> {
        let guard = self;
        let line = unsafe {
            read_config_word(guard.bus, guard.slot, guard.function, OFFSET_INTERRUPT_LINE) as u8
        };
        match line {
            0..=15 => Some(line),
            0xFF => None,
            _ => None,
        }
    }

    pub fn interrupt_pin(&self) -> InterruptPin {
        self.interrupt_pin
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

// BARs
impl PciDevice {
    const OFFSET_BAR0: u8 = 0x10;
    const OFFSET_BAR1: u8 = 0x14;
    const OFFSET_BAR2: u8 = 0x18;
    const OFFSET_BAR3: u8 = 0x1C;
    const OFFSET_BAR4: u8 = 0x20;
    const OFFSET_BAR5: u8 = 0x24;

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
        unsafe { read_config_double_word(self.bus(), self.slot(), self.function(), bar_offset) }
    }

    fn write_bar_raw(&self, bar_offset: u8, value: u32) {
        unsafe {
            write_config_double_word(self.bus(), self.slot(), self.function(), bar_offset, value)
        }
    }
}

// plain getters
impl PciDevice {
    #[inline]
    pub fn bus(&self) -> u8 {
        self.bus
    }

    #[inline]
    pub fn slot(&self) -> u8 {
        self.slot
    }

    #[inline]
    pub fn function(&self) -> u8 {
        self.function
    }

    #[inline]
    pub fn vendor(&self) -> u16 {
        self.vendor
    }

    #[inline]
    pub fn device(&self) -> u16 {
        self.device
    }

    #[inline]
    pub fn is_multi_function(&self) -> bool {
        self.multi_function
    }

    #[inline]
    pub fn header_type(&self) -> PciHeaderType {
        self.header_type
    }
}
