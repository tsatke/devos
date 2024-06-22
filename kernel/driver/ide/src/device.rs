use core::fmt::Debug;

use filesystem::BlockDevice;
use x86_64::instructions::interrupts::without_interrupts;

use crate::command::Command;
use crate::drive::IdeDrive;
use crate::Status;

#[derive(Debug, Clone)]
pub struct IdeBlockDevice {
    ide_drive: IdeDrive,
}

enum AccessMode<'a> {
    Read(&'a mut [u8]),
    Write(&'a [u8]),
}

impl IdeBlockDevice {
    fn access_disk(&self, sector: usize, access_mode: AccessMode) -> Result<usize, <IdeBlockDevice as BlockDevice>::Error> {
        match &access_mode {
            AccessMode::Read(buf) => assert_eq!(buf.len(), self.sector_size()),
            AccessMode::Write(buf) => assert_eq!(buf.len(), self.sector_size()),
        }

        let mut buffer = [0_u16; 256];

        if let AccessMode::Write(buf) = access_mode {
            buf.iter()
                .copied()
                .array_chunks::<2>()
                .map(u16::from_be_bytes)
                .enumerate()
                .for_each(|(i, v)| buffer[i] = v);
        }

        let lba = sector;
        let sector_count = 1;

        let drive_num = self.ide_drive.drive_num();
        let mut channel = self.ide_drive.channel();
        unsafe {
            channel
                .ports
                .drive_select
                .write((0x40 + drive_num) | ((lba >> 24) & 0x0F) as u8);
            channel.ports.features.write(0);
            channel.ports.sector_count.write(sector_count);
            channel.ports.lba_lo.write(lba as u8);
            channel.ports.lba_mid.write((lba >> 8) as u8);
            channel.ports.lba_hi.write((lba >> 16) as u8);
            channel.write_command(Command::ReadSectors);
            channel.disable_irq();
            channel.wait_for_not_busy();
            without_interrupts(|| {
                channel.wait_for_ready();
                while !channel.status().contains(Status::DATA_READY) {}
                match access_mode {
                    AccessMode::Read(_) => {
                        for b in &mut buffer {
                            *b = channel.ports.data.read();
                        }
                    }
                    AccessMode::Write(_) => {
                        for &b in &buffer {
                            channel.ports.data.write(b);
                        }
                    }
                }
            });
        }

        match access_mode {
            AccessMode::Read(buf) => {
                buf.copy_from_slice(unsafe { buffer.as_slice().align_to::<u8>().1 });
                Ok(buf.len())
            }
            AccessMode::Write(buf) => {
                // flush the cache
                channel.write_command(Command::FlushCache);
                channel.poll_on_status(|status| {
                    status.contains(Status::READY) && !status.contains(Status::BUSY)
                });
                Ok(buf.len())
            }
        }
    }
}

impl From<IdeDrive> for IdeBlockDevice {
    fn from(value: IdeDrive) -> Self {
        Self { ide_drive: value }
    }
}

impl BlockDevice for IdeBlockDevice {
    type Error = ();

    fn sector_size(&self) -> usize {
        512
    }

    fn sector_count(&self) -> usize {
        self.ide_drive.sector_count() as usize
    }

    fn read_sector(&self, sector: usize, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.access_disk(sector, AccessMode::Read(buf))
    }

    fn write_sector(&mut self, sector: usize, buf: &[u8]) -> Result<usize, Self::Error> {
        self.access_disk(sector, AccessMode::Write(buf))
    }
}
