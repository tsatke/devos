use crate::ip::IpPacket;
use crate::Packet;
use core::marker::PhantomData;
use thiserror::Error;

pub struct UdpDatagram<'a> {
    _lifetime: PhantomData<&'a ()>,
}

impl Packet for UdpDatagram<'_> {
    fn wire_size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadUdpPacketError {}

impl<'a> TryFrom<IpPacket<'a>> for UdpDatagram<'a> {
    type Error = ReadUdpPacketError;

    fn try_from(_packet: IpPacket<'a>) -> Result<Self, Self::Error> {
        todo!()
    }
}
