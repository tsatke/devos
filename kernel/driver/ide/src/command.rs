#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Command {
    Nop = 0x00,
    DeviceReset = 0x08,
    ReadSectors = 0x20,
    ReadSectorsNoRetry = 0x21,
    ReadLong = 0x22,
    ReadLongNoRetry = 0x23,
    WriteSectors = 0x30,
    WriteSectorsNoRetry = 0x31,
    WriteLong = 0x32,
    WriteLongNoRetry = 0x33,
    FormatTrack = 0x50,
    ReadMultiple = 0xC4,
    WriteMultiple = 0xC5,
    FlushCache = 0xE7,
    Identify = 0xEC,
}

impl From<Command> for u8 {
    fn from(c: Command) -> u8 {
        c as u8
    }
}
