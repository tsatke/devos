#![no_std]

// Ported and adapted from https://wiki.osdev.org/Intel_Ethernet_i217 , thanks a lot!

use bit_field::BitField;
use core::ptr::NonNull;
use pci::{PciDevice, PciStandardHeaderDevice};
use x86_64::instructions::port::Port;
use x86_64::structures::paging::PhysFrame;
use x86_64::{PhysAddr, VirtAddr};

pub const PCI_VENDOR_ID: u16 = 0x8086;
pub const PCI_DEVICE_ID_VM: u16 = 0x100E;
pub const PCI_DEVICE_ID_I217: u16 = 0x153A;
pub const PCI_DEVICE_ID_82577LM: u16 = 0x10EA;

const REG_CTRL: u16 = 0x0000;
const REG_STATUS: u16 = 0x0008;
const REG_EEPROM: u16 = 0x0014;
const REG_CTRL_EXT: u16 = 0x0018;
const REG_IMASK: u16 = 0x00D0;
const REG_RCTRL: u16 = 0x0100;
const REG_RXDESCLO: u16 = 0x2800;
const REG_RXDESCHI: u16 = 0x2804;
const REG_RXDESCLEN: u16 = 0x2808;
const REG_RXDESCHEAD: u16 = 0x2810;
const REG_RXDESCTAIL: u16 = 0x2818;

const REG_TCTRL: u16 = 0x0400;
const REG_TXDESCLO: u16 = 0x3800;
const REG_TXDESCHI: u16 = 0x3804;
const REG_TXDESCLEN: u16 = 0x3808;
const REG_TXDESCHEAD: u16 = 0x3810;
const REG_TXDESCTAIL: u16 = 0x3818;

const REG_RDTR: u16 = 0x2820;
const REG_RXDCTL: u16 = 2828;
const REG_RADV: u16 = 0x282C;
const REG_RSRPD: u16 = 0x2C00;

const REG_TIPG: u16 = 0x0410;

/// End of packet
const CMD_EOP: u8 = 1 << 0;
/// Insert FCS
const CMD_IFCS: u8 = 1 << 1;
/// Insert Checksum
const CMD_IC: u8 = 1 << 2;
/// Report Status
const CMD_RS: u8 = 1 << 3;
/// Report Packet Sent
const CMD_RPS: u8 = 1 << 4;
/// VLAN Packet Enable
const CMD_VLE: u8 = 1 << 6;
/// Interrupt Delay Enable
const CMD_IDE: u8 = 1 << 7;

pub struct E1000 {
    base: BaseAddress,
    has_eeprom: bool,
    mac: [u8; 6],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum BaseAddress {
    Io(u16),
    Memory(NonNull<()>),
}

impl E1000 {
    pub fn new(pci_device: PciStandardHeaderDevice) -> Self {
        let bar = pci_device.bar0();
        let is_io_bar = bar.get_bit(0);
        let base = if is_io_bar {
            BaseAddress::Io(bar.get_bits(2..) as u16)
        } else {
            assert!(!bar.get_bit(2), "64-bit BARs are not supported yet");

            // map the physical address into memory
            let phys_addr = PhysAddr::new(bar.get_bits(4..) as u64);
            let phys_frame = PhysFrame::containing_address(phys_addr);
            let vaddr = VirtAddr::new(0x3222_2222); // TODO: make dynamic

            todo!("map the physical address into memory");

            BaseAddress::Memory(address)
        };

        let mut res = Self {
            base,
            has_eeprom: false,
            mac: [0; 6],
        };
        res.has_eeprom = res.detect_eeprom();
        res
    }

    fn detect_eeprom(&mut self) -> bool {
        self.write_command(REG_EEPROM, 0x01);
        for _ in 0..10000 {
            if self.read_command(REG_EEPROM) & 0x10 > 0 {
                return true;
            }
        }

        false
    }

    pub fn read_eeprom(&mut self, addr: u8) -> u16 {
        let mut tmp: u32;

        let (write_shift, read_shift) = if self.has_eeprom { (8, 2) } else { (4, 1) };

        self.write_command(REG_EEPROM, 1 | ((addr as u32) << write_shift));

        loop {
            tmp = self.read_command(REG_EEPROM);
            if tmp & (1 << read_shift) > 0 {
                break;
            }
        }

        (tmp >> 16 & 0xFFFF) as u16
    }

    pub fn write_command(&mut self, addr: u16, value: u32) {
        match self.base {
            BaseAddress::Io(iobase) => {
                let mut address_port = Port::<u32>::new(iobase);
                let mut value_port = Port::<u32>::new(iobase + 0x04);

                unsafe {
                    address_port.write(addr as u32);
                    value_port.write(value);
                }
            }
            BaseAddress::Memory(membase) => {
                let mut ptr = membase.as_ptr() as *mut u32;
                unsafe {
                    ptr.add(addr as usize).write_volatile(value);
                }
            }
        }
    }

    pub fn read_command(&mut self, addr: u16) -> u32 {
        match self.base {
            BaseAddress::Io(iobase) => {
                let mut address_port = Port::<u32>::new(iobase);
                let mut value_port = Port::<u32>::new(iobase + 0x04);

                unsafe {
                    address_port.write(addr as u32);
                    value_port.read()
                }
            }
            BaseAddress::Memory(membase) => {
                let mut ptr = membase.as_ptr() as *mut u32;
                unsafe { ptr.add(addr as usize).read_volatile() }
            }
        }
    }
}
