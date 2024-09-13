use crate::driver::pci::PciError;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PciDeviceClass {
    Unclassified,
    MassStorageController(MassStorageSubClass),
    NetworkController(NetworkSubClass),
    DisplayController(DisplaySubClass),
    MultimediaController,
    MemoryController,
    Bridge(BridgeSubClass),
    SimpleCommunicationController,
    BaseSystemPeripheral,
    InputDeviceController,
    DockingStation,
    Processor,
    SerialBusController(SerialBusSubClass),
    // WirelessController,
    // IntelligentController,
    // SatelliteCommunicationController,
    // EncryptionController,
    // SignalProcessingController,
    // ProcessingAccelerator,
    // NonEssentialInstrumentation,
    // CoProcessor,
    // UnassignedClass,
}

impl TryFrom<u16> for PciDeviceClass {
    type Error = PciError;

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        let class = (v >> 8) as u8;
        let sub = v as u8;
        Ok(match class {
            0x00 => Self::Unclassified,
            0x01 => Self::MassStorageController(MassStorageSubClass::try_from(sub)?),
            0x02 => Self::NetworkController(NetworkSubClass::try_from(sub)?),
            0x03 => Self::DisplayController(DisplaySubClass::try_from(sub)?),
            0x04 => Self::MultimediaController,
            0x05 => Self::MemoryController,
            0x06 => Self::Bridge(BridgeSubClass::try_from(sub)?),
            0x07 => Self::SimpleCommunicationController,
            0x08 => Self::BaseSystemPeripheral,
            0x09 => Self::InputDeviceController,
            0x0A => Self::DockingStation,
            0x0B => Self::Processor,
            0x0C => Self::SerialBusController(SerialBusSubClass::try_from(sub)?),
            _ => return Err(PciError::UnknownPciDeviceClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DisplaySubClass {
    VGACompatibleController,
    XGAController,
    NoVGA3DController,
    Other,
}

impl TryFrom<u8> for DisplaySubClass {
    type Error = PciError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x00 => Self::VGACompatibleController,
            0x01 => Self::XGAController,
            0x02 => Self::NoVGA3DController,
            0x80 => Self::Other,
            _ => return Err(PciError::UnknownDisplaySubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SerialBusSubClass {
    FireWireController,
    ACCESSBusController,
    SSA,
    USBController,
    FibreChannel,
    SMBusController,
    InfiniBandController,
    IPMIInterface,
    SERCOSInterface,
    CANbusController,
    Other,
}

impl TryFrom<u8> for SerialBusSubClass {
    type Error = PciError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x0 => Self::FireWireController,
            0x1 => Self::ACCESSBusController,
            0x2 => Self::SSA,
            0x3 => Self::USBController,
            0x4 => Self::FibreChannel,
            0x5 => Self::SMBusController,
            0x6 => Self::InfiniBandController,
            0x7 => Self::IPMIInterface,
            0x8 => Self::SERCOSInterface,
            0x9 => Self::CANbusController,
            0x80 => Self::Other,
            _ => return Err(PciError::UnknownSerialBusSubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MassStorageSubClass {
    SCSIBusController,
    IDEController,
    FloppyDiskController,
    IPIBusController,
    RAIDController,
    ATAController,
    SerialATAController,
    SerialAttachedSCSIController,
    NonVolatileMemoryController,
    Other,
}

impl TryFrom<u8> for MassStorageSubClass {
    type Error = PciError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x00 => Self::SCSIBusController,
            0x01 => Self::IDEController,
            0x02 => Self::FloppyDiskController,
            0x03 => Self::IPIBusController,
            0x04 => Self::RAIDController,
            0x05 => Self::ATAController,
            0x06 => Self::SerialATAController,
            0x07 => Self::SerialAttachedSCSIController,
            0x08 => Self::NonVolatileMemoryController,
            0x80 => Self::Other,
            _ => return Err(PciError::UnknownMassStorageSubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NetworkSubClass {
    EthernetController,
    TokenRingController,
    FDDIController,
    ATMController,
    ISDNController,
    WorldFipController,
    PICMG214MultiComputingController,
    InfinibandController,
    FabricController,
    Other,
}

impl TryFrom<u8> for NetworkSubClass {
    type Error = PciError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x00 => Self::EthernetController,
            0x01 => Self::TokenRingController,
            0x02 => Self::FDDIController,
            0x03 => Self::ATMController,
            0x04 => Self::ISDNController,
            0x05 => Self::WorldFipController,
            0x06 => Self::PICMG214MultiComputingController,
            0x07 => Self::InfinibandController,
            0x08 => Self::FabricController,
            0x80 => Self::Other,
            _ => return Err(PciError::UnknownNetworkSubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BridgeSubClass {
    HostBridge,
    ISABridge,
    EISABridge,
    MCABridge,
    PCI2PCIBridge,
    PCMCIABridge,
    NuBusBridge,
    CardBusBridge,
    RACEwayBridge,
    InfiniBand2PCIBridge,
    Other,
}

impl TryFrom<u8> for BridgeSubClass {
    type Error = PciError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0x00 => Self::HostBridge,
            0x01 => Self::ISABridge,
            0x02 => Self::EISABridge,
            0x03 => Self::MCABridge,
            0x04 | 0x09 => Self::PCI2PCIBridge,
            0x05 => Self::PCMCIABridge,
            0x06 => Self::NuBusBridge,
            0x07 => Self::CardBusBridge,
            0x08 => Self::RACEwayBridge,
            0x0A => Self::InfiniBand2PCIBridge,
            0x80 => Self::Other,
            _ => return Err(PciError::UnknownBridgeSubClass(v)),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InterruptPin {
    None,
    INTA,
    INTB,
    INTC,
    INTD,
}

impl TryFrom<u8> for InterruptPin {
    type Error = PciError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0 => Self::None,
            1 => Self::INTA,
            2 => Self::INTB,
            3 => Self::INTC,
            4 => Self::INTD,
            _ => return Err(PciError::UnknownInterruptPin(v)),
        })
    }
}
