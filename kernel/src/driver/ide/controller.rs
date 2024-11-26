use crate::driver::ide::channel::IdeChannel;
use crate::driver::ide::drive::IdeDrive;
use crate::driver::ide::{is_bit_set, register_ide_block_device, IdeBlockDevice};
use crate::driver::pci::PciDevice;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::{Debug, Formatter};
use spin::{Mutex, RwLock};
use thiserror::Error;

pub struct IdeController {
    _device: Weak<Mutex<PciDevice>>,

    primary: Arc<RwLock<IdeChannel>>,
    secondary: Arc<RwLock<IdeChannel>>,
    interrupt_pin: u8,
    interrupt_line: Option<u8>,

    pub drives: Vec<IdeDrive>,
}

impl IdeController {
    pub fn probe(device: &PciDevice) -> bool {
        device.class == 0x01 && device.subclass == 0x01
    }

    pub fn init(device: Weak<Mutex<PciDevice>>) -> Result<(), Box<dyn Error>> {
        let ide_controller = IdeController::try_from(device)?;
        for drive in ide_controller.drives {
            register_ide_block_device(IdeBlockDevice::from(drive))?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum TryFromPciDeviceError {
    #[error("device is not connected")]
    DeviceDisconnected,
}

impl TryFrom<Weak<Mutex<PciDevice>>> for IdeController {
    type Error = TryFromPciDeviceError;

    fn try_from(device: Weak<Mutex<PciDevice>>) -> Result<Self, Self::Error> {
        let device = device
            .upgrade()
            .ok_or(TryFromPciDeviceError::DeviceDisconnected)?;
        Ok(IdeController::from(device))
    }
}

impl From<Arc<Mutex<PciDevice>>> for IdeController {
    fn from(value: Arc<Mutex<PciDevice>>) -> Self {
        let device = value.lock();
        assert!(IdeController::probe(&device));

        let prog_if = device.prog;
        let (primary_ctrlbase, primary_iobase) = if is_bit_set(prog_if as u64, 0) {
            (
                device.base_addresses[1].read() as u16,
                device.base_addresses[0].read() as u16,
            )
        } else {
            (0x3F6, 0x1F0)
        };

        let (secondary_ctrlbase, secondary_iobase) = if is_bit_set(prog_if as u64, 2) {
            (
                device.base_addresses[3].read() as u16,
                device.base_addresses[2].read() as u16,
            )
        } else {
            (0x376, 0x170)
        };

        let bus_master_ide = device.base_addresses[4].read();
        let primary_master_base = bus_master_ide as u16;
        let secondary_master_base = (bus_master_ide >> 16) as u16;

        let mut primary_channels =
            IdeChannel::new(primary_ctrlbase, primary_iobase, primary_master_base);
        let mut secondary_channels =
            IdeChannel::new(secondary_ctrlbase, secondary_iobase, secondary_master_base);
        unsafe {
            // disable interrupts
            primary_channels.disable_irq();
            secondary_channels.disable_irq();
        }

        let primary_channel = Arc::new(RwLock::new(primary_channels));
        let secondary_channel = Arc::new(RwLock::new(secondary_channels));
        let mut drives = vec![];
        for (chan, drive) in [
            (primary_channel.clone(), 0xA0),
            (primary_channel.clone(), 0xB0),
            (secondary_channel.clone(), 0xA0),
            (secondary_channel.clone(), 0xB0),
        ] {
            if let Ok(drive) = IdeDrive::new(chan, drive) {
                drives.push(drive);
            }
        }

        IdeController {
            _device: Arc::downgrade(&value),

            primary: primary_channel,
            secondary: secondary_channel,
            interrupt_pin: device.interrupt_pin,
            interrupt_line: if (0..=15).contains(&device.interrupt_line) {
                Some(device.interrupt_line)
            } else {
                None
            },
            drives,
        }
    }
}

impl Debug for IdeController {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IDEController")
            .field("primary", &self.primary)
            .field("secondary", &self.secondary)
            .field("interrupt pin", &self.interrupt_pin)
            .field("interrupt line", &self.interrupt_line)
            .finish()
    }
}
