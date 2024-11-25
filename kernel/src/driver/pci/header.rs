use crate::driver::pci::PciError;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PciHeaderType {
    Standard = 0x00,
    Pci2PciBridge = 0x01,
    CardBusBridge = 0x02,
}

impl TryFrom<u8> for PciHeaderType {
    type Error = PciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Standard,
            0x01 => Self::Pci2PciBridge,
            0x02 => Self::CardBusBridge,
            _ => return Err(PciError::UnknownHeaderType(value)),
        })
    }
}
