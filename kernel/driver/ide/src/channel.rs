use crate::command::Command;
use crate::{Error, Status};
use alloc::format;
use core::fmt::{Debug, Formatter};
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

#[allow(dead_code)] // a lot of fields are unused, but they exist according to spec, so we keep them
pub struct IdeChannel {
    ctrlbase: u16,
    alternate_status: PortReadOnly<u8>,
    device_control: PortWriteOnly<u8>,
    drive_address: PortReadOnly<u8>,
    iobase: u16,
    pub ports: ChannelsLBA28DataPorts,
    bmide: u16,
    master_ports: ChannelsLBA28DataPorts,
}

impl IdeChannel {
    pub fn new(ctrlbase: u16, iobase: u16, bus_master_ide: u16) -> Self {
        IdeChannel {
            ctrlbase,
            alternate_status: PortReadOnly::new(ctrlbase),
            device_control: PortWriteOnly::new(ctrlbase),
            drive_address: PortReadOnly::new(ctrlbase + 1),
            iobase,
            ports: ChannelsLBA28DataPorts::new(iobase),
            bmide: bus_master_ide,
            master_ports: ChannelsLBA28DataPorts::new(bus_master_ide),
        }
    }

    pub fn write_command(&mut self, cmd: Command) {
        unsafe {
            self.ports.command.write(cmd.into());
        }
    }

    /// Writes the iNIEN bit to the device control port.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it writes to a port,
    /// which could have side effects that violate memory safety.
    pub unsafe fn disable_irq(&mut self) {
        self.device_control.write(2);
    }

    pub fn status(&mut self) -> Status {
        unsafe { Status::from_bits_truncate(self.ports.status.read()) }
    }

    pub fn error(&mut self) -> Error {
        unsafe { Error::from_bits_truncate(self.ports.error.read()) }
    }

    pub fn wait_for_ready(&mut self) {
        self.poll_on_status(|s| s.contains(Status::READY));
    }

    pub fn wait_for_not_busy(&mut self) {
        for _ in 0..16 {
            let _ = self.status();
        }
        self.poll_on_status(|s| !s.contains(Status::BUSY));
    }

    pub fn ctrlbase(&self) -> u16 {
        self.ctrlbase
    }

    pub fn iobase(&self) -> u16 {
        self.iobase
    }

    pub fn poll_on_status<F>(&mut self, f: F)
    where
        F: Fn(Status) -> bool,
    {
        self.poll(IdeChannel::status, f)
    }

    pub fn poll<P, F, T>(&mut self, p: P, f: F)
    where
        P: Fn(&mut Self) -> T,
        F: Fn(T) -> bool,
    {
        loop {
            let t = p(self);
            if f(t) {
                break;
            }
        }
    }
}

impl Debug for IdeChannel {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IDEChannel")
            .field("iobase", &format!("{:#X}", &self.iobase))
            .field("ctrlbase", &format!("{:#X}", &self.ctrlbase))
            .field("bmide", &format!("{:#X}", &self.bmide))
            .finish()
    }
}

pub struct ChannelsLBA28DataPorts {
    pub data: Port<u16>,
    pub error: PortReadOnly<u8>,
    pub features: PortWriteOnly<u8>,
    pub sector_count: Port<u8>,
    pub lba_lo: Port<u8>,
    pub lba_mid: Port<u8>,
    pub lba_hi: Port<u8>,
    pub drive_select: Port<u8>,
    pub status: PortReadOnly<u8>,
    pub command: PortWriteOnly<u8>,
}

impl ChannelsLBA28DataPorts {
    pub fn new(iobase: u16) -> Self {
        Self {
            data: Port::new(iobase),
            error: PortReadOnly::new(iobase + 1),
            features: PortWriteOnly::new(iobase + 1),
            sector_count: Port::new(iobase + 2),
            lba_lo: Port::new(iobase + 3),
            lba_mid: Port::new(iobase + 4),
            lba_hi: Port::new(iobase + 5),
            drive_select: Port::new(iobase + 6),
            status: PortReadOnly::new(iobase + 7),
            command: PortWriteOnly::new(iobase + 7),
        }
    }
}
