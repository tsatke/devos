use crate::driver::pci::raw::{
    read_config_half_word, read_config_word, OFFSET_BAR0, OFFSET_BAR1, OFFSET_BAR2, OFFSET_BAR3, OFFSET_BAR4,
    OFFSET_BAR5, OFFSET_BIST, OFFSET_CLASS, OFFSET_COMMAND, OFFSET_DEVICE_ID,
    OFFSET_HEADER_TYPE, OFFSET_INTERRUPT_LINE, OFFSET_INTERRUPT_PIN, OFFSET_PROG_IF, OFFSET_REVISION_ID,
    OFFSET_STATUS, OFFSET_SUBCLASS, OFFSET_VENDOR_ID,
};
use crate::driver::pci::register::{BaseAddressRegister, PciRegister};
use bitflags::bitflags;
use core::fmt::{Debug, Display, Formatter};

pub struct PciDevice {
    bus: u8,
    slot: u8,
    function: u8,
    vendor_id: u16,
    device_id: u16,
    status: PciRegister<u16>,
    command: PciRegister<u16>,
    rev: u8,
    prog: u8,
    subclass: u8,
    class: u8,
    header_type: u8,
    bist: PciRegister<u8>,
    base_addresses: [BaseAddressRegister; 6],
    interrupt_pin: PciRegister<u8>,
    interrupt_line: PciRegister<u8>,
}

impl Debug for PciDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PciDevice")
            .field("bus", &self.bus)
            .field("slot", &self.slot)
            .field("function", &self.function)
            .field("vendor_id", &self.vendor_id)
            .field("device_id", &self.device_id)
            .field("class", &self.class)
            .field("subclass", &self.subclass)
            .field("prog", &self.prog)
            .field("rev", &self.rev)
            .field("header_type", &self.header_type)
            .finish_non_exhaustive()
    }
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
                rev: read_config_half_word(bus, slot, function, OFFSET_REVISION_ID),
                prog: read_config_half_word(bus, slot, function, OFFSET_PROG_IF),
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
                interrupt_pin: PciRegister::new(bus, slot, function, OFFSET_INTERRUPT_PIN),
                interrupt_line: PciRegister::new(bus, slot, function, OFFSET_INTERRUPT_LINE),
            }
        }
    }

    pub fn class(&self) -> u8 {
        self.class
    }

    pub fn subclass(&self) -> u8 {
        self.subclass
    }

    pub fn status(&self) -> Status {
        Status::from_bits_truncate(self.status.read())
    }

    pub fn bist(&self) -> BIST {
        BIST::from_bits_truncate(self.bist.read())
    }

    pub fn enable_bus_mastering(&mut self) {
        let mut command = self.command.read();
        command |= 1 << 2;
        self.command.write(command);
    }

    pub fn bus(&self) -> u8 {
        self.bus
    }

    pub fn slot(&self) -> u8 {
        self.slot
    }

    pub fn function(&self) -> u8 {
        self.function
    }

    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    pub fn device_id(&self) -> u16 {
        self.device_id
    }

    pub fn command(&self) -> &PciRegister<u16> {
        &self.command
    }

    pub fn rev(&self) -> u8 {
        self.rev
    }

    pub fn prog(&self) -> u8 {
        self.prog
    }

    pub fn header_type(&self) -> u8 {
        self.header_type
    }

    pub fn base_addresses(&self) -> &[BaseAddressRegister; 6] {
        &self.base_addresses
    }

    pub fn base_addresses_mut(&mut self) -> &mut [BaseAddressRegister; 6] {
        &mut self.base_addresses
    }

    pub fn interrupt_pin(&self) -> &PciRegister<u8> {
        &self.interrupt_pin
    }

    pub fn interrupt_line(&self) -> &PciRegister<u8> {
        &self.interrupt_line
    }
}

bitflags! {
    pub struct Status: u16 {
        const DETECTED_PARITY_ERROR = 1 << 15;
        const SIGNALED_SYSTEM_ERROR = 1 << 14;
        const RECEIVED_MASTER_ABORT = 1 << 13;
        const RECEIVED_TARGET_ABORT = 1 << 12;
        const SIGNALED_TARGET_ABORT = 1 << 11;
        const DEVSEL_TIMING = 1 << 10 | 1 << 9;
        const MASTER_DATA_PARITY_ERROR = 1 << 8 ;
        const FAST_BACK_TO_BACK_CAPABLE = 1 << 7;
        const MHZ66_CAPABLE = 1 << 5;
        const CAPABILITIES_LIST = 1 << 4;
        const INTERRUPT = 1 << 3;
    }
}

bitflags! {
    pub struct BIST: u8 {
        const BIST_CAPABLE = 1 << 7;
        const START_BIST = 1 << 6;
        const COMPLETION_CODE = (1 << 4) - 1;
    }
}
