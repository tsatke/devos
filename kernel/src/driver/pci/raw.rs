use virtio_drivers::transport::pci::bus::{ConfigurationAccess, DeviceFunction};
use x86_64::instructions::port::Port;

use crate::driver::pci::device::{PciDevice, ProbeResult};

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

pub const OFFSET_VENDOR_ID: u8 = 0x00;
pub const OFFSET_DEVICE_ID: u8 = 0x02;
pub const OFFSET_SUBSYSTEM_VENDOR_ID: u8 = 0x2C;
pub const OFFSET_SUBSYSTEM_ID: u8 = 0x2E;
pub const OFFSET_STATUS: u8 = 0x06;
pub const OFFSET_COMMAND: u8 = 0x04;
pub const OFFSET_REVISION_ID: u8 = 0x08;
pub const OFFSET_PROG_IF: u8 = 0x09;
pub const OFFSET_SUBCLASS: u8 = 0x0A;
pub const OFFSET_CLASS: u8 = 0x0B;
pub const OFFSET_HEADER_TYPE: u8 = 0x0E;
pub const OFFSET_BIST: u8 = 0x0F;
pub const OFFSET_BAR0: u8 = 0x10;
pub const OFFSET_BAR1: u8 = 0x14;
pub const OFFSET_BAR2: u8 = 0x18;
pub const OFFSET_BAR3: u8 = 0x1C;
pub const OFFSET_BAR4: u8 = 0x20;
pub const OFFSET_BAR5: u8 = 0x24;
pub const OFFSET_INTERRUPT_LINE: u8 = 0x3C;
pub const OFFSET_INTERRUPT_PIN: u8 = 0x3D;

pub unsafe fn iterate_all() -> impl Iterator<Item = PciDevice> {
    (0..=u8::MAX)
        .flat_map(|bus| (0_u8..32).map(move |slot| (bus, slot)))
        .flat_map(|(bus, slot)| {
            match PciDevice::probe(bus, slot, 0) {
                ProbeResult::Exists => 0_u8..1,
                ProbeResult::MultiFunction => 0..8,
                ProbeResult::None => 0..0,
            }
            .map(move |function| (bus, slot, function))
        })
        .filter_map(
            |(bus, slot, function)| match PciDevice::probe(bus, slot, function) {
                ProbeResult::Exists | ProbeResult::MultiFunction => unsafe {
                    Some(PciDevice::new(bus, slot, function))
                },
                ProbeResult::None => None,
            },
        )
}

pub unsafe fn read_config_double_word(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    unsafe {
        u32::from(read_config_word(bus, slot, function, offset))
            | (u32::from(read_config_word(bus, slot, function, offset + 2)) << 16)
    }
}

pub unsafe fn read_config_word(bus: u8, slot: u8, function: u8, offset: u8) -> u16 {
    #[cfg(debug_assertions)]
    assert_eq!(
        offset & 1,
        0,
        "can not read unaligned word, use read_config_half_word instead"
    );

    let mut config_address = Port::<u32>::new(CONFIG_ADDRESS);
    let mut config_data = Port::<u32>::new(CONFIG_DATA);

    let mut address: u32 = 0;
    address |= 1 << 31; // enable bit
    address |= u32::from(bus) << 16;
    address |= u32::from(slot) << 11;
    address |= u32::from(function) << 8;
    address |= u32::from(offset) & 0xFC;
    unsafe { config_address.write(address) };

    unsafe {
        let i = config_data.read();
        (i >> ((offset & 2) * 8) & 0xFFFF) as u16
    }
}

pub unsafe fn read_config_half_word(bus: u8, slot: u8, function: u8, offset: u8) -> u8 {
    let word = unsafe { read_config_word(bus, slot, function, offset & (!1)) };
    if offset & 1 > 0 {
        return (word >> 8) as u8;
    }
    (word & 0x00FF) as u8
}

pub unsafe fn write_config_double_word(bus: u8, slot: u8, function: u8, offset: u8, value: u32) {
    unsafe {
        write_config_word(bus, slot, function, offset, (value & 0x0000_FFFF) as u16);
        write_config_word(bus, slot, function, offset + 2, (value >> 16) as u16);
    }
}

pub unsafe fn write_config_word(bus: u8, slot: u8, function: u8, offset: u8, value: u16) {
    #[cfg(debug_assertions)]
    assert_eq!(
        offset & 1,
        0,
        "can not write unaligned word, use write_config_half_word instead"
    );

    let mut config_address = Port::<u32>::new(CONFIG_ADDRESS);
    let mut config_data = Port::<u32>::new(CONFIG_DATA);

    let mut address: u32 = 0;
    address |= 1 << 31; // enable bit
    address |= u32::from(bus) << 16;
    address |= u32::from(slot) << 11;
    address |= u32::from(function) << 8;
    address |= u32::from(offset) & 0xFC;
    unsafe { config_address.write(address) };

    unsafe {
        let mut i = config_data.read();
        i &= !(0xFFFF << ((offset & 2) * 8));
        i |= u32::from(value) << ((offset & 2) * 8);
        config_data.write(i);
    }
}

pub unsafe fn write_config_half_word(bus: u8, slot: u8, function: u8, offset: u8, value: u8) {
    let mut word = unsafe { read_config_word(bus, slot, function, offset & (!1)) };
    if offset & 1 > 0 {
        word &= 0x00FF;
        word |= u16::from(value) << 8;
    } else {
        word &= 0xFF00;
        word |= u16::from(value);
    }
    unsafe { write_config_word(bus, slot, function, offset & (!1), word) };
}

pub struct PortCam;

impl ConfigurationAccess for PortCam {
    fn read_word(&self, device_function: DeviceFunction, register_offset: u8) -> u32 {
        let bus = device_function.bus;
        let slot = device_function.device;
        let function = device_function.function;

        unsafe { read_config_double_word(bus, slot, function, register_offset) }
    }

    fn write_word(&mut self, device_function: DeviceFunction, register_offset: u8, data: u32) {
        let bus = device_function.bus;
        let slot = device_function.device;
        let function = device_function.function;

        unsafe { write_config_double_word(bus, slot, function, register_offset, data) }
    }

    unsafe fn unsafe_clone(&self) -> Self {
        Self
    }
}
