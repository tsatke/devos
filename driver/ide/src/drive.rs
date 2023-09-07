use crate::channel::IdeChannel;
use crate::command::Command;
use crate::{is_bit_set, Status, UDMAMode};
use alloc::format;
use alloc::sync::Arc;
use core::fmt::{Debug, Display, Formatter};
use spin::{RwLock, RwLockWriteGuard};
use x86_64::instructions::interrupts;

#[derive(Clone)]
pub struct IdeDrive {
    channel: Arc<RwLock<IdeChannel>>,

    ctrlbase: u16,
    iobase: u16,
    drive: u8,

    exists: bool,

    // The following block consists of the identify_sector and then values
    // that were read from it.
    identify_sector: [u16; 256],
    supported_udma_modes: UDMAMode,
    active_udma_mode: UDMAMode,
    sector_count: u64,
}

impl Display for IdeDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "IDEDrive[ctrlbase={:#X} iobase={:#X} drive={:#X}]",
            self.ctrlbase, self.iobase, self.drive
        )
    }
}

impl Debug for IdeDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IDEDrive")
            .field("channel", &self.channel.read())
            .field("drive", &format!("{:#X}", self.drive))
            .field("exists", &self.exists)
            .field("sector count", &self.sector_count)
            .field("udma support", &self.supported_udma_modes)
            .field("active udma", &self.active_udma_mode)
            .finish()
    }
}

impl IdeDrive {
    pub fn new(channel: Arc<RwLock<IdeChannel>>, drive: u8) -> Result<Self, IdentifyError> {
        let ctrlbase = channel.read().ctrlbase();
        let iobase = channel.read().iobase();
        let mut drive = IdeDrive {
            channel,
            ctrlbase,
            iobase,
            drive,
            exists: false,
            identify_sector: [0; 256],
            supported_udma_modes: UDMAMode::empty(),
            active_udma_mode: UDMAMode::empty(),
            sector_count: 0,
        };
        drive.exists = drive.identify()?;
        Ok(drive)
    }
}

impl IdeDrive {
    pub fn exists(&self) -> bool {
        self.exists
    }

    pub fn ctrlbase(&self) -> u16 {
        self.channel.read().ctrlbase()
    }

    pub fn iobase(&self) -> u16 {
        self.channel.read().iobase()
    }

    pub fn drive_num(&self) -> u8 {
        self.drive
    }
}

pub struct IdentifyError;

impl IdeDrive {
    pub(crate) fn channel_mut(&mut self) -> RwLockWriteGuard<IdeChannel> {
        self.channel.write()
    }

    fn identify(&mut self) -> Result<bool, IdentifyError> {
        let mut channel = self.channel.write();
        unsafe {
            channel.ports.drive_select.write(self.drive);

            channel.ports.lba_lo.write(0);
            channel.ports.lba_mid.write(0);
            channel.ports.lba_hi.write(0);

            channel.write_command(Command::Identify);
            let status = channel.status();
            if status.bits() == 0 {
                return Ok(false);
            }

            while channel.status().contains(Status::BUSY) {
                // do nothing
            }
            if channel.ports.lba_mid.read() != 0 || channel.ports.lba_hi.read() != 0 {
                return Ok(false);
            }
            loop {
                let status = channel.status();
                if status.contains(Status::ERROR) {
                    return Err(IdentifyError);
                }
                if status.contains(Status::DATA_READY) {
                    break;
                }
            }

            channel.wait_for_not_busy();

            interrupts::disable();
            channel.wait_for_ready();
            channel.ports.command.write(Command::ReadSectors.into());

            for i in 0..self.identify_sector.len() {
                self.identify_sector[i] = channel.ports.data.read();
            }
            interrupts::enable();

            let udma_indicator = self.identify_sector[88];
            self.active_udma_mode = UDMAMode::from_bits_truncate((udma_indicator >> 8) as u8);
            self.supported_udma_modes = UDMAMode::from_bits_truncate(udma_indicator as u8);

            if self.is_lba48_supported() {
                self.sector_count = self.identify_sector[100] as u64
                    | ((self.identify_sector[101] as u64) << 16)
                    | ((self.identify_sector[102] as u64) << 32)
                    | ((self.identify_sector[103] as u64) << 48)
            } else {
                self.sector_count =
                    self.identify_sector[60] as u64 | ((self.identify_sector[61] as u64) << 16)
            }
        }
        Ok(true)
    }

    pub fn is_lba48_supported(&self) -> bool {
        is_bit_set(self.identify_sector[83] as u64, 10)
    }

    pub fn supported_udma_modes(&self) -> UDMAMode {
        self.supported_udma_modes
    }

    pub fn active_udma_mode(&self) -> UDMAMode {
        self.active_udma_mode
    }
}
