///! xHCI register definitions according to
///! [the spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf)

use core::fmt::{Debug, Formatter};
use core::ptr::NonNull;
use volatile::access::NoAccess;
use volatile::access::ReadOnly;
use volatile::access::ReadWrite;
use volatile::{VolatileFieldAccess, VolatilePtr};
use x86_64::VirtAddr;


#[repr(C)]
#[derive(Debug)]
pub struct Registers<'a> {
    pub capabilities: VolatilePtr<'a, Capabilities>,
    pub operational: VolatilePtr<'a, Operational>,
    pub port: VolatilePtr<'a, Port>,
    pub runtime: VolatilePtr<'a, Runtime>,
}

impl Registers<'_> {
    pub fn new(base: VirtAddr) -> Self {
        let capabilities = unsafe { VolatilePtr::new(NonNull::new(base.as_mut_ptr::<Capabilities>()).unwrap()) };

        let caplength = capabilities.caplength().read();
        let operational_base = base + caplength as u64;
        assert!(base + size_of::<Capabilities>() < operational_base, "capabilities registers should not overlap into operational registers");
        let operational = unsafe { VolatilePtr::new(NonNull::new(operational_base.as_mut_ptr::<Operational>()).unwrap()) };

        let port_base = operational_base + 0x400_usize;
        assert!(operational_base + size_of::<Operational>() < port_base, "operational registers should not overlap into port registers");
        let port = unsafe { VolatilePtr::new(NonNull::new(port_base.as_mut_ptr::<Port>()).unwrap()) };

        let rtsoff = capabilities.rtsoff().read();
        let runtime_base = base + rtsoff as u64;
        let runtime = unsafe { VolatilePtr::new(NonNull::new(runtime_base.as_mut_ptr::<Runtime>()).unwrap()) };

        Self { capabilities, operational, port, runtime }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Capabilities {
    /// This register is used as an offset to add to register base to find the beginning of
    /// the Operational Register Space.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=381)
    #[access(ReadOnly)]
    caplength: u8,
    #[access(NoAccess)]
    rsvd: u8,
    /// This is a two-byte register containing a BCD encoding of the xHCI specification
    /// revision number supported by this host controller. The most significant byte of
    /// this register represents a major revision and the least significant byte contains
    /// the minor revision extensions. e.g. 0100h corresponds to xHCI version 1.0.0, or
    /// 0110h corresponds to xHCI version 1.1.0, etc.
    ///
    /// **Note**: Pre-release versions of the xHC shall declare the specific version of the xHCI that
    /// it was implemented against. e.g. 0090h = version 0.9.0.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=381)
    #[access(ReadOnly)]
    hciversion: u16,
    /// [`HcsParams1`]
    #[access(ReadOnly)]
    hcsparams1: HcsParams1,
    /// [`HcsParams2`]
    #[access(ReadOnly)]
    hcsparams2: HcsParams2,
    /// [`HcsParams3`]
    #[access(ReadOnly)]
    hcsparams3: HcsParams3,
    /// [`HccParams1`]
    #[access(ReadOnly)]
    hccparams1: HccParams1,
    /// [`DbOff`]
    #[access(ReadOnly)]
    dboff: DbOff,
    /// This register defines the offset of the xHCI Runtime Registers from the Base.
    ///
    /// **Note**: Normally the Runtime Register Space is 32-byte aligned, however if virtualization
    /// is supported by the xHC (either through IOV or VTIO) then it shall be PAGESIZE
    /// aligned. e.g. If the PAGESIZE = 4K and the Runtime Register Space is positioned
    /// at a 1 page offset from the Base, then this register shall report 0000 1000h.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=388)
    #[access(ReadOnly)]
    rtsoff: u32,
    /// [`HccParams2`]
    #[access(ReadOnly)]
    hccparms2: HccParams2,
    /// [`VtiosOff`]
    vtiosoff: VtiosOff,
}

/// # Structural Parameters 1
/// This register defines basic structural parameters supported by this xHC
/// implementation: Number of Device Slots support, Interrupters, Root Hub ports,
/// etc.
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HcsParams1(u32);

impl HcsParams1 {
    /// # Number of Ports (MaxPorts)
    /// This field specifies the maximum Port Number value, i.e. the
    /// highest numbered Port Register Set that are addressable in the Operational Register Space
    /// (refer to Table 5-18). Valid values are in the range of 1h to FFh.
    ///
    /// The value in this field shall reflect the maximum Port Number value assigned by an xHCI
    /// Supported Protocol Capability, described in section 7.2. Software shall refer to these capabilities
    /// to identify whether a specific Port Number is valid, and the protocol supported by the
    /// associated Port Register Set.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub fn max_ports(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    ///# Number of Device Slots (MaxSlots)
    /// This field specifies the maximum number of Device
    /// Context Structures and Doorbell Array entries this host controller can support. Valid values are
    /// in the range of 1 to 255. The value of `0` is reserved.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
    pub fn max_device_slots(&self) -> u8 {
        self.0 as u8
    }

    /// # Number of Interrupters (MaxIntrs)
    /// This field specifies the number of Interrupters implemented
    /// on this host controller. Each Interrupter may be allocated to a MSI or MSI-X vector and controls
    /// its generation and moderation.
    ///
    /// The value of this field determines how many Interrupter Register Sets are addressable in the
    /// Runtime Register Space (refer to section 5.5). Valid values are in the range of 1h to 400h. A `0` in
    /// this field is undefined.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
    pub fn max_interrupters(&self) -> u16 {
        ((self.0 >> 8) & ((1 << 9) - 1)) as u16
    }
}

impl Debug for HcsParams1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HcsParams1")
            .field("max_ports", &self.max_ports())
            .field("max_device_slots", &self.max_device_slots())
            .field("max_interrupters", &self.max_interrupters())
            .finish()
    }
}

/// # Structural Parameters 2
/// This register defines additional xHC structural parameters.
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HcsParams2(u32);

impl HcsParams2 {
    /// # Isochronous Scheduling Threshold (IST)
    /// Default = implementation dependent. The value in
    /// this field indicates to system software the minimum distance (in time) that it is required to stay
    /// ahead of the host controller while adding TRBs, in order to have the host controller process
    /// them at the correct time. The value shall be specified in terms of number of
    /// frames/microframes.
    ///
    /// If bit \[3\] of IST is cleared to '0', software can add a TRB no later than IST\[2:0\] Microframes
    /// before that TRB is scheduled to be executed.
    ///
    /// If bit \[3\] of IST is set to '1', software can add a TRB no later than IST\[2:0\] Frames before that TRB
    /// is scheduled to be executed.
    ///
    /// Refer to Section 4.14.2 for details on how software uses this information for scheduling
    /// isochronous transfers.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub fn ist(&self) -> u8 {
        (self.0 & 0x111) as u8
    }

    /// # Event Ring Segment Table Max (ERST Max)
    /// Default = implementation dependent. Valid values
    /// are 0 – 15. This field determines the maximum value supported the Event Ring Segment Table
    /// Base Size registers (5.5.2.3.1), where:
    /// The maximum number of Event Ring Segment Table entries = 2 ERST Max.
    /// e.g. if the ERST Max = 7, then the xHC Event Ring Segment Table(s) supports up to 128 entries,
    /// 15 then 32K entries, etc.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub fn erst_max(&self) -> u16 {
        ((self.0 >> 3) & ((1 << 4) - 1)) as u16
    }

    /// # Max Scratchpad Buffers (Max Scratchpad Bufs Hi)
    /// Default = implementation dependent. This
    /// field indicates the high order 5 bits of the number of Scratchpad Buffers system software shall
    /// reserve for the xHC. Refer to section 4.20 for more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn max_scratchpad_bufs_hi(&self) -> u8 {
        ((self.0 >> 20) & ((1 << 5) - 1)) as u8
    }

    /// # Max Scratchpad Buffers (Max Scratchpad Bufs Lo)
    /// Default = implementation dependent. Valid
    /// values for Max Scratchpad Buffers (Hi and Lo) are 0-1023. This field indicates the low order 5
    /// bits of the number of Scratchpad Buffers system software shall reserve for the xHC. Refer to
    /// section 4.20 for more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn max_scratchpad_bufs_lo(&self) -> u8 {
        (self.0 >> 27) as u8
    }

    /// See [`max_scratchpad_bufs_hi`](HcsParams2::max_scratchpad_bufs_hi) and
    /// [`max_scratchpad_bufs_lo`](HcsParams2::max_scratchpad_bufs_lo).
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn max_scratchpad_bufs(&self) -> u16 {
        ((self.max_scratchpad_bufs_hi() as u16) << 8) | self.max_scratchpad_bufs_lo() as u16
    }

    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn scratchpad_restore(&self) -> bool {
        (self.0 & (1 << 26)) != 0
    }
}

impl Debug for HcsParams2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HcsParams2")
            .field("ist", &self.ist())
            .field("erst_max", &self.erst_max())
            .field("max_scratchpad_bufs_hi", &self.max_scratchpad_bufs_hi())
            .field("max_scratchpad_bufs_lo", &self.max_scratchpad_bufs_lo())
            .field("max_scratchpad_bufs", &self.max_scratchpad_bufs())
            .field("scratchpad_restore", &self.scratchpad_restore())
            .finish()
    }
}

/// # Structural Parameters 3
/// This register defines link exit latency related structural parameters.
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HcsParams3(u32);

impl HcsParams3 {
    /// # U1 Device Exit Latency
    /// Worst case latency to transition a root hub Port Link State (PLS) from
    /// U1 to U0. Applies to all root hub ports.
    /// The following are permissible values:
    ///
    /// | Value | Description |
    /// |-------|-------------|
    /// | 00h | Zero |
    /// | 01h | Less than 1 μs |
    /// | 02h | Less than 2 μs. |
    /// | ... | ... |
    /// | 0Ah | Less than 10 μs. |
    /// | 0B-FFh | Reserved |
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn u1_device_exit_latency(&self) -> u8 {
        self.0 as u8
    }

    /// # U2 Device Exit Latency
    /// Worst case latency to transition from U2 to U0. Applies to all root hub
    /// ports.
    /// The following are permissible values:
    ///
    /// | Value | Description |
    /// |-------|-------------|
    /// | 0000h | Zero |
    /// | 0001h | Less than 1 μs. |
    /// | 0002h | Less than 2 μs. |
    /// | ... | ... |
    /// | 07FFh | Less than 2047 μs.|
    /// | 0800-FFFFh | Reserved |
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=385)
    pub fn u2_device_exit_latency(&self) -> u16 {
        (self.0 >> 16) as u16
    }
}

impl Debug for HcsParams3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HcsParams3")
            .field("u1_device_exit_latency", &self.u1_device_exit_latency())
            .field("u2_device_exit_latency", &self.u2_device_exit_latency())
            .finish()
    }
}

/// # Capability Parameters 1
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=385)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HccParams1(u32);

impl HccParams1 {
    /// # 64-bit Addressing Capability (AC64)
    /// This flag documents the addressing range capability of
    /// this implementation. The value of this flag determines whether the xHC has implemented the
    /// high order 32 bits of 64 bit register and data structure pointer fields. Values for this flag have the
    /// following interpretation:
    ///
    /// | Value | Description |
    /// |-------|-------------|
    /// | 0 | 32-bit address memory pointers implemented |
    /// | 1 | 64-bit address memory pointers implemented |
    ///
    /// spec!(386)
    ///
    /// If 32-bit address memory pointers are implemented, the xHC shall ignore the high order 32 bits
    /// of 64 bit data structure pointer fields, and system software shall ignore the high order 32 bits of
    /// 64 bit xHC registers.
    ///
    /// This is not tightly coupled with the USBBASE address register mapping control. The 64-bit Addressing Capability
    /// (AC64) flag indicates whether the host controller can generate 64-bit addresses as a master. The USBBASE
    /// register indicates the host controller only needs to decode 32-bit addresses as a slave
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn ac64(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// # BW Negotiation Capability (BNC)
    /// This flag identifies whether the xHC has implemented the
    /// Bandwidth Negotiation. Values for this flag have the following interpretation:
    ///
    /// | Value | Description                       |
    /// |-------|-----------------------------------|
    /// | 0     | BW Negotiation not implemented    |
    /// | 1     | BW Negotiation implemented        |
    ///
    /// Refer to section 4.16 for more information on Bandwidth Negotiation
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn bnc(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// # Context Size (CSZ)
    /// If this bit is set to `1`, then the xHC uses 64 byte Context data structures. If
    /// this bit is cleared to `0`, then the xHC uses 32 byte Context data structures.
    ///
    /// **Note**: This flag does not apply to Stream Contexts.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn csz(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// # Port Power Control (PPC)
    /// This flag indicates whether the host controller implementation
    /// includes port power control. A `1` in this bit indicates the ports have port power switches. A `0` in
    /// this bit indicates the port do not have port power switches. The value of this flag affects the
    /// functionality of the PP flag in each port status and control register (refer to Section 5.4.8).
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn ppc(&self) -> bool {
        (self.0 & (1 << 3)) != 0
    }

    /// # Port Indicators (PIND)
    /// This bit indicates whether the xHC root hub ports support port indicator
    /// control. When this bit is a `1`, the port status and control registers include a read/writeable field
    /// for controlling the state of the port indicator. Refer to Section 5.4.8 for definition of the Port
    /// Indicator Control field.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn pind(&self) -> bool {
        (self.0 & (1 << 4)) != 0
    }

    /// # Light HC Reset Capability (LHRC)
    /// This flag indicates whether the host controller implementation
    /// supports a Light Host Controller Reset. A `1` in this bit indicates that Light Host Controller Reset is
    /// supported. A `0` in this bit indicates that Light Host Controller Reset is not supported. The value
    /// of this flag affects the functionality of the Light Host Controller Reset (LHCRST) flag in the
    /// USBCMD register (refer to Section 5.4.1).
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn lhrc(&self) -> bool {
        (self.0 & (1 << 5)) != 0
    }

    /// # Latency Tolerance Messaging Capability (LTC)
    /// This flag indicates whether the host controller
    /// implementation supports Latency Tolerance Messaging (LTM). A `1` in this bit indicates that LTM
    /// is supported. A `0` in this bit indicates that LTM is not supported. Refer to section 4.13.1 for more
    /// information on LTM.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn ltc(&self) -> bool {
        (self.0 & (1 << 6)) != 0
    }

    /// # No Secondary SID Support (NSS)
    /// This flag indicates whether the host controller
    /// implementation supports Secondary Stream IDs. A `1` in this bit indicates that Secondary Stream
    /// ID decoding is not supported. A `0` in this bit indicates that Secondary Stream ID decoding is
    /// supported. (refer to Sections 4.12.2 and 6.2.3).
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn nss(&self) -> bool {
        (self.0 & (1 << 7)) != 0
    }

    /// # Parse All Event Data (PAE)
    /// This flag indicates whether the host controller implementation
    /// Parses all Event Data TRBs while advancing to the next TD after a Short Packet, or it skips all but
    /// the first Event Data TRB. A `1` in this bit indicates that all Event Data TRBs are parsed. A `0` in this
    /// bit indicates that only the first Event Data TRB is parsed (refer to section 4.10.1.1).
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn pae(&self) -> bool {
        (self.0 & (1 << 8)) != 0
    }

    /// # Stopped - Short Packet Capability (SPC)
    /// This flag indicates that the host controller
    /// implementation is capable of generating a Stopped - Short Packet Completion Code. Refer to
    /// section 4.6.9 for more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn spc(&self) -> bool {
        (self.0 & (1 << 9)) != 0
    }

    /// # Stopped EDTLA Capability (SEC)
    /// This flag indicates that the host controller implementation
    /// Stream Context support a Stopped EDTLA field. Refer to sections 4.6.9, 4.12, and 6.4.4.1 for more
    /// information.
    /// Stopped EDTLA Capability support (i.e. SEC = '1') shall be mandatory for all xHCI 1.1 and xHCI 1.2
    /// compliant xHCs.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn sec(&self) -> bool {
        (self.0 & (1 << 10)) != 0
    }

    /// # Contiguous Frame ID Capability (CFC)
    /// This flag indicates that the host controller
    /// implementation is capable of matching the Frame ID of consecutive Isoch TDs. Refer to section
    /// 4.11.2.5 for more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn cfc(&self) -> bool {
        (self.0 & (1 << 11)) != 0
    }

    /// # Maximum Primary Stream Array Size (MaxPSASize)
    /// This fields identifies the maximum size
    /// Primary Stream Array that the xHC supports. The Primary Stream Array size = 2MaxPSASize+1. Valid
    /// MaxPSASize values are 0 to 15, where `0` indicates that Streams are not supported.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn max_psa_size(&self) -> u8 {
        ((self.0 >> 12) & ((1 << 4) - 1)) as u8
    }

    /// # xHCI Extended Capabilities Pointer (xECP)
    /// This field indicates the existence of a capabilities list.
    /// The value of this field indicates a relative offset, in 32-bit words, from Base to the beginning of
    /// the first extended capability.
    /// For example, using the offset of Base is 1000h and the xECP value of 0068h, we can calculated
    /// the following effective address of the first extended capability:
    /// 1000h + (0068h << 2) -> 1000h + 01A0h -> 11A0h
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn xecp(&self) -> u16 {
        (self.0 >> 16) as u16
    }
}

impl Debug for HccParams1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HccParams1")
            .field("ac64", &self.ac64())
            .field("bnc", &self.bnc())
            .field("csz", &self.csz())
            .field("ppc", &self.ppc())
            .field("pind", &self.pind())
            .field("lhrc", &self.lhrc())
            .field("ltc", &self.ltc())
            .field("nss", &self.nss())
            .field("pae", &self.pae())
            .field("spc", &self.spc())
            .field("sec", &self.sec())
            .field("cfc", &self.cfc())
            .field("max_psa_size", &self.max_psa_size())
            .field("xecp", &self.xecp())
            .finish()
    }
}

/// This register defines the offset of the Doorbell Array base address from the
/// Base.
///
/// **Note**: Normally the Doorbell Array is Dword aligned, however if virtualization is
/// supported by the xHC (either through IOV or VTIO) then it shall be PAGESIZE
/// aligned. e.g. If the PAGESIZE = 4K (1000h), and the Doorbell Array is positioned
/// at a 3 page offset from the Base, then this register shall report 0000 3000h.
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct DbOff(u32);

impl DbOff {
    /// # Doorbell Array Offset - RO
    /// Default = implementation dependent. This field defines the offset
    /// in Dwords of the Doorbell Array base address from the Base (i.e. the base address of the xHCI
    /// Capability register address space).
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=388)
    pub fn offset(&self) -> u32 {
        self.0 >> 2
    }
}

impl Debug for DbOff {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DbOff")
            .field("offset", &self.offset())
            .finish()
    }
}

/// This register defines optional capabilities supported by the xHCI.
/// The default values for all fields in this register are implementation dependent.
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HccParams2(u32);

impl HccParams2 {
    /// # U3 Entry Capability (U3C) - RO
    /// This bit indicates whether the xHC Root Hub ports support port
    /// Suspend Complete notification. When this bit is '1', PLC shall be asserted on any transition of
    /// PLS to the U3 State. Refer to section 4.15.1 for more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub fn u3c(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// # Configure Endpoint Command Max Exit Latency Too Large Capability (CMC) - RO
    /// This bit
    /// indicates whether a Configure Endpoint Command is capable of generating a Max Exit Latency
    /// Too Large Capability Error. When this bit is '1', a Max Exit Latency Too Large Capability Error
    /// may be returned by a Configure Endpoint Command. When this bit is '0', a Max Exit Latency Too
    /// Large Capability Error shall not be returned by a Configure Endpoint Command. This capability
    /// is enabled by the CME flag in the USBCMD register. Refer to sections 4.23.5.2 and 5.4.1 for more
    /// information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub fn cmc(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// # Force Save Context Capability (FSC) - RO
    /// This bit indicates whether the xHC supports the
    /// Force Save Context Capability. When this bit is '1', the Save State operation shall save any
    /// cached Slot, Endpoint, Stream or other Context information to memory. Refer to
    /// Implementation Note “FSC and Context handling by Save and Restore”, and sections 4.23.2 and
    /// 5.4.1 for more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub fn fsc(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// # Compliance Transition Capability (CTC) - RO
    /// This bit indicates whether the xHC USB3 Root
    /// Hub ports support the Compliance Transition Enabled (CTE) flag. When this bit is `1`, USB3 Root
    /// Hub port state machine transitions to the Compliance substate shall be explicitly enabled
    /// software. When this bit is `0`, USB3 Root Hub port state machine transitions to the Compliance
    /// substate are automatically enabled. Refer to section 4.19.1.2.4.1 for more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn ctc(&self) -> bool {
        (self.0 & (1 << 3)) != 0
    }

    /// # Large ESIT Payload Capability (LEC) - RO
    /// This bit indicates whether the xHC supports ESIT
    /// Payloads greater than 48K bytes. When this bit is `1`, ESIT Payloads greater than 48K bytes are
    /// supported. When this bit is `0`, ESIT Payloads greater than 48K bytes are not supported. Refer to
    /// section 6.2.3.8 for more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn lec(&self) -> bool {
        (self.0 & (1 << 4)) != 0
    }

    /// # Configuration Information Capability (CIC) - RO
    /// This bit indicates if the xHC supports
    /// extended Configuration Information. When this bit is 1, the Configuration Value, Interface
    /// Number, and Alternate Setting fields in the Input Control Context are supported. When this bit is
    /// 0, the extended Input Control Context fields are not supported. Refer to section 6.2.5.1 for more
    /// information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn cic(&self) -> bool {
        (self.0 & (1 << 5)) != 0
    }

    /// # Extended TBC Capability78 (ETC) - RO
    /// This bit indicates if the TBC field in an Isoch TRB
    /// supports the definition of Burst Counts greater than 65535 bytes. When this bit is `1`, the
    /// Extended EBC capability is supported by the xHC. When this bit is `0`, it is not. Refer to section
    /// 4.11.2.3 for more information.
    ///
    /// The Extended TBC Capability (ETC) was added to enable support for Transfer Burst Count (TBC) values greater
    /// than 4, which are required to fully support SSP Isoch bandwidths.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn etc(&self) -> bool {
        (self.0 & (1 << 6)) != 0
    }

    /// # Extended TBC TRB Status Capability (ETC_TSC) - RO
    /// This bit indicates if the TBC/TRBSts field
    /// in an Isoch TRB indicates additional information regarding TRB in the TD. When this bit is `1`, the
    /// Isoch TRB TD Size/TBC field presents TBC value and TBC/TRBSts field presents the TRBSts
    /// value. When this bit is `0` then the ETC/ETE values defines the TD Size/TBC field and TBC/RsvdZ
    /// field. This capability shall be enabled only if LEC = `1` and ETC=`1`. Refer to section 4.11.2.3 for
    /// more information.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn etc_tsc(&self) -> bool {
        (self.0 & (1 << 7)) != 0
    }

    /// # Get/Set Extended Property Capability (GSC) – RO
    /// This bit indicates support for the Set
    /// Extended Property and Get Extended Property commands. When this bit is `1`, the xHC supports
    /// the Get Extended Property and Set Extended Property commands defined in section 4.6.17 and
    /// section 4.6.18. When this bit is `0`, the xHC does not support the Get Extended Property and Set
    /// Extended Property commands and the xHC does not support any of the associated Extended
    /// Capabilities.
    /// This bit shall only be set to `1` if the xHC supports one or more extended capabilities that
    /// require the Get Extended Property and Set Extended Property commands.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn gsc(&self) -> bool {
        (self.0 & (1 << 8)) != 0
    }

    /// # Virtualization Based Trusted I/O Capability (VTC) – RO
    /// This bit when set to 1, indicates that
    /// the xHC supports the Virtualization based Trusted IO (VTIO) Capability. When this bit is 0, the
    /// VTIO Capability is not supported. This capability is enabled by the VTIOE flag in the USBCMD
    /// register.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn vtc(&self) -> bool {
        (self.0 & (1 << 9)) != 0
    }
}

impl Debug for HccParams2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HccParams2")
            .field("u3c", &self.u3c())
            .field("cmc", &self.cmc())
            .field("fsc", &self.fsc())
            .field("ctc", &self.ctc())
            .field("lec", &self.lec())
            .field("cic", &self.cic())
            .field("etc", &self.etc())
            .field("etc_tsc", &self.etc_tsc())
            .field("gsc", &self.gsc())
            .field("vtc", &self.vtc())
            .finish()
    }
}

/// This register defines the offset of the xHCI VTIO Registers from the Base.
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct VtiosOff(u32);

impl VtiosOff {
    /// # VTIO Register Space Offset – RO
    /// Default = implementation dependent. This field defines the
    /// offset in 4 KByte offset of the VTIO Registers from the Base. i.e. VTIO Register Base = Base +
    /// VTIO Register Space Offset.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=391)
    pub fn offset(&self) -> u32 {
        self.0 >> 12
    }
}

impl Debug for VtiosOff {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VtiosOff")
            .field("offset", &self.offset())
            .finish()
    }
}

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

#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Port {
    // TODO: implement
}

#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Runtime {
    #[access(ReadWrite)]
    mfindex: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Interrupter {
    #[access(ReadWrite)]
    iman: u32,
    #[access(ReadWrite)]
    imod: u32,
    #[access(ReadWrite)]
    erstsz: u64,
    #[access(ReadWrite)]
    erstba: u64,
    #[access(ReadWrite)]
    erdp: u64,
}

