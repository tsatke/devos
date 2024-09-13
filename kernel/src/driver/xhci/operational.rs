use bitflags::bitflags;
use core::fmt;
use core::fmt::{Debug, Formatter};
use volatile::access::{ReadOnly, ReadWrite};
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
/// | Offset | Mnemonic | Register Name | Section |
/// |--------|----------|---------------|---------|
/// | 00h | USBCMD | USB Command | 5.4.1 |
/// | 04h | USBSTS | USB Status | 5.4.2 |
/// | 08h | PAGESIZE | Page Size | 5.4.3 |
/// | 0C-13h | RsvdZ |
/// | 14h | DNCTRL | Device Notification Control | 5.4.4 |
/// | 18h | CRCR | Command Ring Control | 5.4.5 |
/// | 20-2Fh | RsvdZ |
/// | 30h | DCBAAP | Device Context Base Address Array Pointer | 5.4.6 |
/// | 38h | CONFIG | Configure | 5.4.7 |
/// | 3C-3FFh | RsvdZ |
/// | 400-13FFh | Port Register Set 1-MaxPorts | 5.4.8, 5.4.9 |
///
/// **Note**: The MaxPorts value in the HCSPARAMS1 register defines the number of Port
/// Register Sets (e.g. PORTSC, PORTPMSC, and PORTLI register sets). The PORTSC,
/// PORTPMSC, and PORTLI register sets are grouped (consecutive Dwords). Refer
/// to their respective sections for their addressing.
///
/// The Offset referenced in Table 5-18 is the offset from the beginning of the
/// Operational Register space.
///
/// The Operational registers are located at a positive offset from the Capabilities
/// Registers (refer to Section 5.3).
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=391)
#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Operational {
    /// [`UsbCmd`]
    #[access(ReadWrite)]
    usbcmd: UsbCmd,
    /// [`UsbSts`]
    #[access(ReadWrite)]
    usbsts: UsbSts,
    /// [`Pagesize`]
    #[access(ReadOnly)]
    pagesize: Pagesize,
    #[access(ReadWrite)]
    dnctrl: DnCtrl,
    #[access(ReadWrite)]
    crcr: u64,
    #[access(ReadWrite)]
    dcbaap: u64,
    #[access(ReadWrite)]
    config: u32,
}

bitflags! {
    /// # USB Command Register
    /// The Command Register indicates the command to be executed by the serial bus
    /// host controller. Writing to the register causes a command to be executed.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=393)
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    pub struct UsbCmd: u32 {
        /// # Run/Stop (R/S) – RW
        /// Default = `0`. `1` = Run. `0` = Stop. When set to a `1`, the xHC proceeds with
        /// execution of the schedule. The xHC continues execution as long as this bit is set to a `1`. When
        /// this bit is cleared to `0`, the xHC completes any current or queued commands or TDs, and any
        /// USB transactions associated with them, then halts.
        ///
        /// Refer to section 5.4.1.1 for more information on how R/S shall be managed.
        ///
        /// The xHC shall halt within 16 ms. after software clears the Run/Stop bit if the above conditions
        /// have been met.
        ///
        /// The HCHalted (HCH) bit in the USBSTS register indicates when the xHC has finished its pending
        /// pipelined transactions and has entered the stopped state. Software shall not write a `1` to this
        /// flag unless the xHC is in the Halted state (i.e. HCH in the USBSTS register is `1`). Doing so may
        /// yield undefined results. Writing a `0` to this flag when the xHC is in the Running state (i.e. HCH =
        /// `0`) and any Event Rings are in the Event Ring Full state (refer to section 4.9.4) may result in lost
        /// events.
        ///
        /// When this register is exposed by a Virtual Function (VF), this bit only controls the run state of
        /// the xHC instance presented by the selected VF. Refer to section 8 for more information.
        ///
        /// After R/S is written with a `0` by software, the xHC completes any current or
        /// queued commands or TDs (and any host initiated transactions on the USB
        /// associated with them), then halts and sets HCH = `1`. The time it takes for the
        /// xHC to halt depends on many things, however if many TDs are queued on
        /// Transfer Rings, then it may take a long time for the xHC to complete all
        /// outstanding work and halt.
        ///
        /// To expedite the xHC halt process, software should ensure the following before
        /// clearing the R/S bit:
        ///
        /// - All endpoints are in the Stopped state or Idle in the Running state, and all
        /// Transfer Events associated with them have been received.
        /// - The Command Transfer Ring is in the Stopped state (CRR = `0`) or Idle (i.e.
        /// the Command Transfer Ring is empty), and all Command Completion
        /// Events associated with them have been received.
        /// Software should apply the following rules to determine when a Busy Transfer
        /// Ring becomes Idle:
        /// - For Isoch endpoints:
        ///     - Wait for a Ring Underrun or Ring Overrun Transfer Event or,
        ///     - Issue a Stop Endpoint Command and wait for the associated
        ///       Command Completion Event.
        /// - For non-Isoch endpoints:
        ///     - If the IOC flag is set in the last TRB on the Transfer Ring, then wait
        ///       for its Transfer Event.
        ///     - If the IOC flag is not set in the last TRB on the Transfer Ring, then
        ///       there will be no Transfer Event generated when the last TRB on
        ///       the ring is completed, so software shall issue a Stop Endpoint
        ///       Command and wait for the associated Command Completion
        ///       Event and Stopped Transfer Events. Refer to section 4.6.9.
        ///
        /// **Note**: Software shall ensure that any pending reset on a USB2 port is completed before
        /// R/S is cleared to `0`.
        ///
        /// **Note**: The xHC is forced to halt within 16 ms. of software clearing the R/S bit to `0`,
        /// irrespective of any queued Transfer or Command Ring activity. If software does
        /// not follow the “halt process” recommendations above, undefined behavior may
        /// occur, e.g. xHC commands or pending USB transactions may be lost, aborted, etc
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=393)
        const RS = 1 << 0;

        /// # Host Controller Reset (HCRST) – RW
        /// Default = `0`. This control bit is used by software to reset
        /// the host controller. The effects of this bit on the xHC and the Root Hub registers are similar to a
        /// Chip Hardware Reset.
        ///
        /// When software writes a `1` to this bit, the Host Controller resets its internal pipelines, timers,
        /// counters, state machines, etc. to their initial value. Any transaction currently in progress on the
        /// USB is immediately terminated. A USB reset shall not be driven on USB2 downstream ports,
        /// however a Hot or Warm Reset shall be initiated on USB3 Root Hub downstream ports.
        ///
        /// Depending on the link state when HCRST is asserted, an xHC implementation may choose to issue a Hot Reset
        /// rather than a Warm Reset to accelerate the USB recovery process.
        ///
        /// PCI Configuration registers are not affected by this reset. All operational registers, including port
        /// registers and port state machines are set to their initial values. Software shall reinitialize the
        /// host controller as described in Section 4.2 in order to return the host controller to an
        /// operational state.
        ///
        /// This bit is cleared to `0` by the Host Controller when the reset process is complete. Software
        /// cannot terminate the reset process early by writing a `0` to this bit and shall not write any xHC
        /// Operational or Runtime registers until while HCRST is `1`. Note, the completion of the xHC reset
        /// process is not gated by the Root Hub port reset process.
        ///
        /// Software shall not set this bit to `1` when the HCHalted (HCH) bit in the USBSTS register is a `0`.
        /// Attempting to reset an actively running host controller may result in undefined behavior.
        /// When this register is exposed by a Virtual Function (VF), this bit only resets the xHC instance
        /// presented by the selected VF. Refer to section 8 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=394)
        const HCRST = 1 << 1;

        /// # Interrupter Enable (INTE) – RW
        /// Default = `0`. This bit provides system software with a means of
        /// enabling or disabling the host system interrupts generated by Interrupters. When this bit is a `1`,
        /// then Interrupter host system interrupt generation is allowed, e.g. the xHC shall issue an interrupt
        /// at the next interrupt threshold if the host system interrupt mechanism (e.g. MSI, MSI-X, etc.) is
        /// enabled. The interrupt is acknowledged by a host system interrupt specific mechanism.
        ///
        /// When this register is exposed by a Virtual Function (VF), this bit only enables the set of
        /// Interrupters assigned to the selected VF. Refer to section 7.7.2 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=394)
        const INTE = 1 << 2;

        /// # Host System Error Enable (HSEE) – RW
        /// Default = `0`. When this bit is a `1`, and the HSE bit in
        /// the USBSTS register is a `1`, the xHC shall assert out-of-band error signaling to the host. The
        /// signaling is acknowledged by software clearing the HSE bit. Refer to section 4.10.2.6 for more
        /// information.
        ///
        /// When this register is exposed by a Virtual Function (VF), the effect of the assertion of this bit on
        /// the Physical Function (PF0) is determined by the VMM. Refer to section 8 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=394)
        const HSEE = 1 << 3;

        /// # Light Host Controller Reset (LHCRST) – RO or RW
        /// Optional normative. Default = `0`. If the Light
        /// HC Reset Capability (LHRC) bit in the HCCPARAMS1 register is `1`, then this flag allows the driver
        /// to reset the xHC without affecting the state of the ports.
        ///
        /// A system software read of this bit as `0` indicates the Light Host Controller Reset has completed
        /// and it is safe for software to re-initialize the xHC. A software read of this bit as a `1` indicates the
        /// Light Host Controller Reset has not yet completed.
        ///
        /// If not implemented, a read of this flag shall always return a `0`.
        /// All registers in the Aux Power well shall maintain the values that had been asserted prior to the
        /// Light Host Controller Reset. Refer to section 4.23.1 for more information.
        ///
        /// When this register is exposed by a Virtual Function (VF), this bit only generates a Light Reset to
        /// the xHC instance presented by the selected VF, e.g. Disable the VFs` device slots and set the
        /// associated VF Run bit to Stopped. Refer to section 8 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const LHCRST = 1 << 7;

        /// # Controller Save State (CSS) - RW
        /// Default = `0`. When written by software with `1` and HCHalted
        /// (HCH) = `1`, then the xHC shall save any internal state (that may be restored by a subsequent
        /// Restore State operation) and if FSC = '1' any cached Slot, Endpoint, Stream, or other Context
        /// information (so that software may save it). When written by software with `1` and HCHalted
        /// (HCH) = `0`, or written with `0`, no Save State operation shall be performed. This flag always
        /// returns `0` when read. Refer to the Save State Status (SSS) flag in the USBSTS register for
        /// information on Save State completion. Refer to section 4.23.2 for more information on xHC
        /// Save/Restore operation. Note that undefined behavior may occur if a Save State operation is
        /// initiated while Restore State Status (RSS) = `1`.
        ///
        /// When this register is exposed by a Virtual Function (VF), this bit only controls saving the state of
        /// the xHC instance presented by the selected VF. Refer to section 8 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const CSS = 1 << 8;

        /// # Controller Restore State (CRS) - RW
        /// Default = `0`. When set to `1`, and HCHalted (HCH) = `1`,
        /// then the xHC shall perform a Restore State operation and restore its internal state. When set to
        /// `1` and Run/Stop (R/S) = `1` or HCHalted (HCH) = `0`, or when cleared to `0`, no Restore State
        /// operation shall be performed. This flag always returns `0` when read. Refer to the Restore State
        /// Status (RSS) flag in the USBSTS register for information on Restore State completion. Refer to
        /// section 4.23.2 for more information. Note that undefined behavior may occur if a Restore State
        /// operation is initiated while Save State Status (SSS) = `1`.
        ///
        /// When this register is exposed by a Virtual Function (VF), this bit only controls restoring the state
        /// of the xHC instance presented by the selected VF. Refer to section 8 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const CRS = 1 << 9;

        /// # Enable Wrap Event (EWE) - RW
        /// Default = `0`. When set to `1`, the xHC shall generate a MFINDEX
        /// Wrap Event every time the MFINDEX register transitions from 03FFFh to 0. When cleared to `0`
        /// no MFINDEX Wrap Events are generated. Refer to section 4.14.2 for more information.
        ///
        /// When this register is exposed by a Virtual Function (VF), the generation of MFINDEX Wrap
        /// Events to VFs shall be emulated by the VMM.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const EWE = 1 << 10;

        /// # Enable U3 MFINDEX Stop (EU3S) - RW
        /// Default = `0`. When set to `1`, the xHC may stop the
        /// MFINDEX counting action if all Root Hub ports are in the U3, Disconnected, Disabled, or
        /// Powered-off state. When cleared to `0` the xHC may stop the MFINDEX counting action if all
        /// Root Hub ports are in the Disconnected, Disabled, Training, or Powered-off state. Refer to
        /// section 4.14.2 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const EU3S = 1 << 11;

        /// # CEM Enable (CME) - RW
        /// Default = '0'. When set to '1', a Max Exit Latency Too Large Capability
        /// Error may be returned by a Configure Endpoint Command. When cleared to '0', a Max Exit
        /// Latency Too Large Capability Error shall not be returned by a Configure Endpoint Command.
        /// This bit is Reserved if CMC = `0`. Refer to section 4.23.5.2.2 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=396)
        const CME = 1 << 13;

        /// # Extended TBC Enable (ETE)
        /// This flag indicates that the host controller implementation is
        /// enabled to support Transfer Burst Count (TBC) values greater that 4 in isoch TDs. When this bit
        /// is `1`, the Isoch TRB TD Size/TBC field presents the TBC value, and the TBC/RsvdZ field is RsvdZ.
        /// When this bit is `0`, the TDSize/TCB field presents the TD Size value, and the TBC/RsvdZ field
        /// presents the TBC value. This bit may be set only if ETC = `1`. Refer to section 4.11.2.3 for more
        /// information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=396)
        const ETE = 1 << 14;

        /// # Extended TBC TRB Status Enable (TSC_EN)
        /// This flag indicates that the host controller
        /// implementation is enabled to support ETC_TSC capability. When this is `1`, TRBSts field in the
        /// TRB updated to indicate if it is last transfer TRB in the TD. This bit may be set only if
        /// ETC_TSC=`1`. Refer to section 4.11.2.3 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=396)
        const TSC_EN = 1 << 15;

        /// # VTIO Enable (VTIOE) – RW
        /// Default = `0`. When set to `1`, XHCI HW will enable its VTIO
        /// capability and begin to use the information provided via that VTIO Registers to determine its
        /// DMA-ID. When cleared to `0`, XHCI HW will use the Primary DMA-ID for all accesses. This bit
        /// may be set only if VTC = `1`.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=396)
        const VTIOE = 1 << 16;
    }
}

bitflags! {
    /// # USB Status Register
    /// This register indicates pending interrupts and various states of the Host
    /// Controller. The status resulting from a transaction on the serial bus is not
    /// indicated in this register. Software sets a bit to `0` in this register by writing a `1`
    /// to it (RW1C). Refer to Section 4.17 for additional information concerning USB
    /// interrupt conditions.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=397)
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    pub struct UsbSts: u32 {
        /// # HCHalted (HCH) – RO
        /// Default = `1`. This bit is a `0` whenever the Run/Stop (R/S) bit is a `1`. The
        /// xHC sets this bit to `1` after it has stopped executing as a result of the Run/Stop (R/S) bit being
        /// cleared to `0`, either by software or by the xHC hardware (e.g. internal error).
        ///
        /// If this bit is '1', then SOFs, microSOFs, or Isochronous Timestamp Packets (ITP) shall not be
        /// generated by the xHC, and any received Transaction Packet shall be dropped.
        ///
        /// When this register is exposed by a Virtual Function (VF), this bit only reflects the Halted state of
        /// the xHC instance presented by the selected VF. Refer to section 8 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=398)
        const HCH = 1 << 0;

        /// # Host System Error (HSE) – RW1C
        /// Default = `0`
        /// The xHC sets this bit to `1` when a serious error
        /// is detected, either internal to the xHC or during a host system access involving the xHC module.
        ///
        /// (In a PCI system, conditions that set this bit to `1` include PCI Parity error, PCI Master Abort, and
        /// PCI Target Abort.) When this error occurs, the xHC clears the Run/Stop (R/S) bit in the USBCMD
        /// register to prevent further execution of the scheduled TDs. If the HSEE bit in the USBCMD
        /// register is a `1`, the xHC shall also assert out-of-band error signaling to the host. Refer to section
        /// 4.10.2.6 for more information.
        ///
        /// When this register is exposed by a Virtual Function (VF), the assertion of this bit affects all VFs
        /// and reflects the Host System Error state of the Physical Function (PF0). Refer to section 8 for
        /// more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=398)
        const HSE = 1 << 2;

        /// # Event Interrupt (EINT) – RW1C
        /// Default = `0`. The xHC sets this bit to `1` when the Interrupt
        /// Pending (IP) bit of any Interrupter transitions from `0` to `1`. Refer to section 7.1.2 for use.
        ///
        /// Software that uses EINT shall clear it prior to clearing any IP flags. A race condition may occur if
        /// software clears the IP flags then clears the EINT flag, and between the operations another IP `0`
        /// to '1' transition occurs. In this case the new IP transition shall be lost.
        ///
        /// When this register is exposed by a Virtual Function (VF), this bit is the logical 'OR' of the IP bits
        /// for the Interrupters assigned to the selected VF. And it shall be cleared to `0` when all associated
        /// interrupter IP bits are cleared, i.e. all the VF`s Interrupter Event Ring(s) are empty. Refer to
        /// section 8 for more information.
        ///
        /// **Note**: The Event Interrupt (EINT) and Port Change Detect (PCD) flags are typically only
        /// used by system software for managing the xHCI when interrupts are disabled or
        /// during an SMI.
        ///
        /// **Note**: The EINT flag does not generate an interrupt, it is simply a logical OR of the IMAN
        /// register IP flag `0` to `1` transitions. As such, it does not need to be cleared to clear
        /// an xHC interrupt.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=398)
        const EINT = 1 << 3;

        /// # Port Change Detect (PCD) – RW1C
        /// Default = `0`. The xHC sets this bit to a `1` when any port has
        /// a change bit transition from a `0` to a `1`.
        ///
        /// This bit is allowed to be maintained in the Aux Power well. Alternatively, it is also acceptable
        /// that on a D3 to D0 transition of the xHC, this bit is loaded with the OR of all of the PORTSC
        /// change bits. Refer to section 4.19.3.
        ///
        /// This bit provides system software an efficient means of determining if there has been Root Hub
        /// port activity. Refer to section 4.15.2.3 for more information.
        ///
        /// When this register is exposed by a Virtual Function (VF), the VMM determines the state of this
        /// bit as a function of the Root Hub Ports associated with the Device Slots assigned to the selected
        /// VF. Refer to section 8 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=398)
        const PCD = 1 << 4;

        /// # Save State Status (SSS) - RO
        /// Default = `0`. When the Controller Save State (CSS) flag in the
        /// USBCMD register is written with `1` this bit shall be set to `1` and remain 1 while the xHC saves
        /// its internal state. When the Save State operation is complete, this bit shall be cleared to `0`.
        /// Refer to section 4.23.2 for more information.
        ///
        /// When this register is exposed by a Virtual Function (VF), the VMM determines the state of this
        /// bit as a function of the saving the state for the selected VF. Refer to section 8 for more
        /// information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const SSS = 1 << 8;

        /// # Restore State Status (RSS) - RO
        /// Default = `0`. When the Controller Restore State (CRS) flag in
        /// the USBCMD register is written with `1` this bit shall be set to `1` and remain 1 while the xHC
        /// restores its internal state. When the Restore State operation is complete, this bit shall be
        /// cleared to `0`. Refer to section 4.23.2 for more information.
        ///
        /// When this register is exposed by a Virtual Function (VF), the VMM determines the state of this
        /// bit as a function of the restoring the state for the selected VF. Refer to section 8 for more
        /// information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const RSS = 1 << 9;

        /// # Save/Restore Error (SRE) - RW1C
        /// Default = `0`. If an error occurs during a Save or Restore
        /// operation this bit shall be set to `1`. This bit shall be cleared to `0` when a Save or Restore
        /// operation is initiated or when written with `1`. Refer to section 4.23.2 for more information.
        ///
        /// When this register is exposed by a Virtual Function (VF), the VMM determines the state of this
        /// bit as a function of the Save/Restore completion status for the selected VF. Refer to section 8
        /// for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const SRE = 1 << 10;

        /// # Controller Not Ready (CNR) – RO
        /// Default = `1`. `0` = Ready and `1` = Not Ready. Software shall
        /// not write any Doorbell or Operational register of the xHC, other than the USBSTS register, until
        /// CNR = `0`. This flag is set by the xHC after a Chip Hardware Reset and cleared when the xHC is
        /// ready to begin accepting register writes. This flag shall remain cleared (`0`) until the next Chip
        /// Hardware Reset.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const CNR = 1 << 11;

        /// # Host Controller Error (HCE) – RO
        /// Default = 0. 0` = No internal xHC error conditions exist and `1`
        /// = Internal xHC error condition. This flag shall be set to indicate that an internal error condition
        /// has been detected which requires software to reset and reinitialize the xHC. Refer to section
        /// 4.24.1 for more information.
        ///
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const HCE = 1 << 12;
    }
}

/// # Page Size – RO
/// Default = Implementation defined. This field defines the page size supported by
/// the xHC implementation. This xHC supports a page size of 2^(n+12) ([`Pagesize::size`]) if bit n ([`Pagesize::size_raw`]) is Set. For example, if
/// bit 0 is Set, the xHC supports 4k byte page sizes.
///
/// For a Virtual Function, this register reflects the page size selected in the System Page Size field
/// of the SR-IOV Extended Capability structure. For the Physical Function 0, this register reflects
/// the implementation dependent default xHC page size.
///
/// Various xHC resources reference PAGESIZE to describe their minimum alignment requirements.
/// The maximum possible page size is 128M.
///
/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Pagesize(u32);

impl Pagesize {
    /// [`Pagesize`]
    pub fn size_raw(&self) -> u32 {
        self.0 & ((1 << 16) - 1)
    }

    /// [`Pagesize`]
    pub fn size(&self) -> u32 {
        1 << (self.size_raw() + 12)
    }
}

impl Debug for Pagesize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pagesize")
            .field("size", &self.size())
            .finish()
    }
}

bitflags! {
    /// # Device Notification Control Register
    /// This register is used by software to enable or disable the reporting of the
    /// reception of specific USB Device Notification Transaction Packets. A Notification
    /// Enable (Nx, where x = 0 to 15) flag is defined for each of the 16 possible de vice
    /// notification types. If a flag is set for a specific notification type, a Device
    /// Notification Event shall be generated when the respective notification packet is
    /// received. After reset all notifications are disabled. Refer to section 6.4.2.7.
    ///
    /// This register shall be written as a Dword. Byte writes produce undefined results.
    ///
    /// ## Notification Enable (N0-N15) – RW
    /// When a Notification Enable bit is set, a Device Notification
    /// Event shall be generated when a Device Notification Transaction Packet is received with the
    /// matching value in the Notification Type field. For example, setting N1 to ‘1’ enables Device
    /// Notification Event generation if a Device Notification TP is received with its Notification Type
    /// field set to ‘1’ (FUNCTION_WAKE), etc.
    ///
    /// **Note**: Of the currently defined USB3 Device Notification Types, only the
    /// FUNCTION_WAKE type should not be handled automatically by the xHC. Only
    /// under debug conditions would software write the DNCTRL register with a value
    /// other than 0002h. Refer to section 8.5.6 in the USB3 specification for more
    /// information on Notification Types. If new Device Notification Types are defined,
    /// software may receive them by setting the respective Notification Enable bit.
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=400)
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    pub struct DnCtrl: u32 {
        /// [`DnCtrl`]
        const N0 = 1 << 0;
        /// [`DnCtrl`]
        const N1 = 1 << 1;
        /// [`DnCtrl`]
        const N2 = 1 << 2;
        /// [`DnCtrl`]
        const N3 = 1 << 3;
        /// [`DnCtrl`]
        const N4 = 1 << 4;
        /// [`DnCtrl`]
        const N5 = 1 << 5;
        /// [`DnCtrl`]
        const N6 = 1 << 6;
        /// [`DnCtrl`]
        const N7 = 1 << 7;
        /// [`DnCtrl`]
        const N8 = 1 << 8;
        /// [`DnCtrl`]
        const N9 = 1 << 9;
        /// [`DnCtrl`]
        const N10 = 1 << 10;
        /// [`DnCtrl`]
        const N11 = 1 << 11;
        /// [`DnCtrl`]
        const N12 = 1 << 12;
        /// [`DnCtrl`]
        const N13 = 1 << 13;
        /// [`DnCtrl`]
        const N14 = 1 << 14;
        /// [`DnCtrl`]
        const N15 = 1 << 15;
    }
}