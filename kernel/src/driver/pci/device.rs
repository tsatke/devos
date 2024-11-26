use crate::driver::pci::raw::{
    read_config_half_word, read_config_word, OFFSET_BAR0, OFFSET_BAR1, OFFSET_BAR2, OFFSET_BAR3,
    OFFSET_BAR4, OFFSET_BAR5, OFFSET_BIST, OFFSET_CLASS, OFFSET_COMMAND, OFFSET_DEVICE_ID,
    OFFSET_HEADER_TYPE, OFFSET_INTERRUPT_LINE, OFFSET_INTERRUPT_PIN, OFFSET_PROG_IF_REVISION_ID,
    OFFSET_STATUS, OFFSET_SUBCLASS, OFFSET_VENDOR_ID,
};
use crate::driver::pci::register::{BaseAddressRegister, PciRegister};
use crate::driver::pci::{Status, BIST};
use core::fmt::Formatter;
use derive_more::Display;

#[derive(Debug)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub status: PciRegister<u16>,
    pub command: PciRegister<u16>,
    pub rev: u8,
    pub prog: u8,
    pub subclass: u8,
    pub class: u8,
    pub header_type: u8,
    pub bist: PciRegister<u8>,
    pub base_addresses: [BaseAddressRegister; 6],
    pub interrupt_pin: u8,
    pub interrupt_line: u8,
}

impl Display for PciDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "id {:04x}:{:04x}, slot {:02x}:{:02x}.{:x}",
            self.vendor_id, self.device_id, self.bus, self.slot, self.function,
        )
    }
}

pub enum ProbeResult {
    Exists,
    MultiFunction,
    None,
}

impl PciDevice {
    pub fn probe(bus: u8, slot: u8, function: u8) -> ProbeResult {
        let vendor_id = unsafe { read_config_word(bus, slot, function, OFFSET_VENDOR_ID) };
        if vendor_id == 0xFFFF {
            return ProbeResult::None;
        }

        let header_type = unsafe { read_config_half_word(bus, slot, function, OFFSET_HEADER_TYPE) };
        if header_type & 0x80 > 0 {
            ProbeResult::MultiFunction
        } else {
            ProbeResult::Exists
        }
    }

    pub(in crate::driver::pci) unsafe fn new(bus: u8, slot: u8, function: u8) -> Self {
        unsafe {
            Self {
                bus,
                slot,
                function,
                vendor_id: read_config_word(bus, slot, function, OFFSET_VENDOR_ID),
                device_id: read_config_word(bus, slot, function, OFFSET_DEVICE_ID),
                status: PciRegister::new(bus, slot, function, OFFSET_STATUS),
                command: PciRegister::new(bus, slot, function, OFFSET_COMMAND),
                rev: read_config_half_word(bus, slot, function, OFFSET_PROG_IF_REVISION_ID),
                prog: read_config_half_word(bus, slot, function, OFFSET_PROG_IF_REVISION_ID),
                subclass: read_config_half_word(bus, slot, function, OFFSET_SUBCLASS),
                class: read_config_half_word(bus, slot, function, OFFSET_CLASS),
                header_type: read_config_half_word(bus, slot, function, OFFSET_HEADER_TYPE),
                bist: PciRegister::new(bus, slot, function, OFFSET_BIST),
                base_addresses: [
                    PciRegister::new(bus, slot, function, OFFSET_BAR0).into(),
                    PciRegister::new(bus, slot, function, OFFSET_BAR1).into(),
                    PciRegister::new(bus, slot, function, OFFSET_BAR2).into(),
                    PciRegister::new(bus, slot, function, OFFSET_BAR3).into(),
                    PciRegister::new(bus, slot, function, OFFSET_BAR4).into(),
                    PciRegister::new(bus, slot, function, OFFSET_BAR5).into(),
                ],
                interrupt_pin: read_config_half_word(bus, slot, function, OFFSET_INTERRUPT_PIN),
                interrupt_line: read_config_half_word(bus, slot, function, OFFSET_INTERRUPT_LINE),
            }
        }
    }

    pub fn status(&self) -> Status {
        Status::from_bits_truncate(self.status.read())
    }

    pub fn bist(&self) -> BIST {
        BIST::from_bits_truncate(self.bist.read())
    }
}
