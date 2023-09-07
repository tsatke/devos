use crate::command::Command;
use crate::drive::IdeDrive;
use crate::Status;
use x86_64::instructions::interrupts::without_interrupts;

#[derive(Debug, Clone)]
pub struct IdeBlockDevice {
    ide_drive: IdeDrive,
}

impl IdeBlockDevice {}

impl From<IdeDrive> for IdeBlockDevice {
    fn from(value: IdeDrive) -> Self {
        Self { ide_drive: value }
    }
}

impl filesystem::BlockDevice for IdeBlockDevice {
    type Error = ();

    fn sector_size(&self) -> usize {
        512
    }

    fn sector_count(&self) -> usize {
        self.ide_drive.sector_count() as usize
    }

    fn read_sector(&self, sector: usize, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut data = [0_u16; 256];

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
                for b in &mut data {
                    *b = channel.ports.data.read();
                }
            });
            channel.poll_on_status(|status| {
                status.contains(Status::READY) && !status.contains(Status::BUSY)
            });
        }

        buf.copy_from_slice(unsafe { data.as_slice().align_to::<u8>().1 });
        Ok(buf.len())
    }

    fn write_sector(&mut self, _block: usize, _buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}
