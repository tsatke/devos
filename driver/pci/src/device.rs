use alloc::sync::Arc;

use spin::RwLock;

use crate::raw::{
    read_config_half_word, read_config_word, OFFSET_BIST, OFFSET_CLASS_SUBCLASS,
    OFFSET_HEADER_TYPE, OFFSET_INTERRUPT_LINE, OFFSET_INTERRUPT_PIN, OFFSET_PROG_IF_REVISION_ID,
    OFFSET_STATUS,
};
use crate::{Error, InterruptPin, PciDeviceClass, PciHeaderType, Status, BIST};

#[derive(Clone, Debug)]
pub struct PciDevice {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
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

impl PciDevice {
    /// Create a new pci device from the given parameters.
    ///
    /// # Safety
    ///
    /// Creating a new pci device is unsafe because this
    /// reads from memory, which could have unintended
    /// effects. Also, the caller has to ensure that this
    /// is only called once for every combination of parameters.
    pub(crate) unsafe fn new(
        bus: u8,
        slot: u8,
        function: u8,
        vendor: u16,
        device: u16,
    ) -> Result<Self, Error> {
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
        let d = PciDevice {
            inner: Arc::new(RwLock::new(Inner {
                bus,
                slot,
                function,
                vendor,
                device,
                header_type,
                multi_function,
                class,
                interrupt_pin,
            })),
        };
        Ok(d)
    }

    pub fn class(&self) -> PciDeviceClass {
        self.inner.read().class
    }

    pub fn prog_if(&self) -> u8 {
        let guard = self.inner.read();
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
        let guard = self.inner.read();
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
        let guard = self.inner.read();
        let status =
            unsafe { read_config_word(guard.bus, guard.slot, guard.function, OFFSET_STATUS) };
        Status::from_bits_truncate(status)
    }

    pub fn bist(&self) -> BIST {
        let guard = self.inner.read();
        let bist = unsafe { read_config_word(guard.bus, guard.slot, guard.function, OFFSET_BIST) };
        BIST::from_bits_truncate(bist as u8)
    }

    pub fn interrupt_line(&self) -> Option<u8> {
        let guard = self.inner.read();
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
        self.inner.read().interrupt_pin
    }
}

// plain getters
impl PciDevice {
    #[inline]
    pub fn bus(&self) -> u8 {
        self.inner.read().bus
    }

    #[inline]
    pub fn slot(&self) -> u8 {
        self.inner.read().slot
    }

    #[inline]
    pub fn function(&self) -> u8 {
        self.inner.read().function
    }

    #[inline]
    pub fn vendor(&self) -> u16 {
        self.inner.read().vendor
    }

    #[inline]
    pub fn device(&self) -> u16 {
        self.inner.read().device
    }

    #[inline]
    pub fn is_multi_function(&self) -> bool {
        self.inner.read().multi_function
    }

    #[inline]
    pub fn header_type(&self) -> PciHeaderType {
        self.inner.read().header_type
    }
}
