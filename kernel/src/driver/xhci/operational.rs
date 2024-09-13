use volatile::access::ReadWrite;
use volatile::VolatileFieldAccess;

/// # Host Controller Operational Registers
/// This section defines the xHCI Operational Registers.
///
/// The base address of this register space is referred to as Operational Base. The
/// Operational Base shall be Dword aligned and is calculated by adding the value
/// of the Capability Registers Length (CAPLENGTH) register (refer to Section 5.3.1)
/// to the Capability Base address. All registers are multiples of 32 bits in length.
///
/// Unless otherwise stated, all registers should be accessed as a 32 -bit width on
/// reads with an appropriate software mask, if needed. A software
/// read/modify/write mechanism should be invoked for partial writes.
///
/// These registers are located at a positive offset from the Capabilities Registers
/// (refer to Section 5.3).
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=391)
#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Operational {
    #[access(ReadWrite)]
    usbcmd: u32,
    #[access(ReadWrite)]
    usbsts: u32,
    #[access(ReadWrite)]
    pagesize: u32,
    #[access(ReadWrite)]
    dnctrl: u32,
    #[access(ReadWrite)]
    crcr: u64,
    #[access(ReadWrite)]
    dcbaap: u64,
    #[access(ReadWrite)]
    config: u32,
}