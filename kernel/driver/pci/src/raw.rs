use crate::PciDevice;
use alloc::vec::Vec;
use x86_64::instructions::port::Port;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

const OFFSET_VENDOR_ID: u8 = 0x00;
const OFFSET_DEVICE: u8 = 0x02;
pub const OFFSET_STATUS: u8 = 0x06;
pub const OFFSET_HEADER_TYPE: u8 = 0x0E;
pub const OFFSET_PROG_IF_REVISION_ID: u8 = 0x08;
pub const OFFSET_CLASS_SUBCLASS: u8 = 0x0A;
pub const OFFSET_BIST: u8 = 0x0F;
pub const OFFSET_INTERRUPT_LINE: u8 = 0x3C;
pub const OFFSET_INTERRUPT_PIN: u8 = 0x3D;

pub unsafe fn iterate_bus(bus: u8, devices: &mut Vec<PciDevice>) {
    for slot in 0..32 {
        // function 0 must be implemented by all devices, so if we don't get
        // a device with function 0, there will not be any devices on other
        // functions for the same bus and slot
        if let Some(dev) = check_device(bus, slot, 0) {
            if dev.is_multi_function() {
                iterate_functions(bus, slot, devices);
            } else {
                devices.push(dev);
            }
        }
    }
}

unsafe fn iterate_functions(bus: u8, slot: u8, devices: &mut Vec<PciDevice>) {
    for function in 0..8 {
        if let Some(dev) = check_device(bus, slot, function) {
            // TODO: if the device is a PCI2PCI bridge, iterate that as bus as well
            devices.push(dev);
        }
    }
}

unsafe fn check_device(bus: u8, slot: u8, function: u8) -> Option<PciDevice> {
    if let Some((vendor, device)) = pci_check_vendor(bus, slot, function) {
        let dev = match PciDevice::new(bus, slot, function, vendor, device) {
            Ok(d) => d,
            Err(_) => {
                return None;
            }
        };
        return Some(dev);
    }
    None
}

unsafe fn pci_check_vendor(bus: u8, slot: u8, function: u8) -> Option<(u16, u16)> {
    let vendor = read_config_word(bus, slot, function, OFFSET_VENDOR_ID);
    if vendor == 0xFFFF {
        return None;
    }

    let device = read_config_word(bus, slot, function, OFFSET_DEVICE);

    Some((vendor, device))
}

pub unsafe fn read_config_double_word(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    read_config_word(bus, slot, function, offset) as u32
        | ((read_config_word(bus, slot, function, offset + 2) as u32) << 16)
}

pub unsafe fn read_config_word(bus: u8, slot: u8, function: u8, offset: u8) -> u16 {
    #[cfg(debug_assertions)]
    if offset & 1 > 0 {
        panic!("can not read unaligned word, use read_config_half_word instead");
    }

    let mut config_address = Port::<u32>::new(CONFIG_ADDRESS);
    let mut config_data = Port::<u32>::new(CONFIG_DATA);

    let mut address: u32 = 0;
    address |= 1 << 31; // enable bit
    address |= (bus as u32) << 16;
    address |= (slot as u32) << 11;
    address |= (function as u32) << 8;
    address |= (offset as u32) & 0xFC;
    config_address.write(address);

    let i = config_data.read();
    (i >> ((offset & 2) * 8) & 0xFFFF) as u16
}

pub unsafe fn read_config_half_word(bus: u8, slot: u8, function: u8, offset: u8) -> u8 {
    let word = read_config_word(bus, slot, function, offset & (!1));
    if offset & 1 > 0 {
        return (word >> 8) as u8;
    }
    word as u8
}
