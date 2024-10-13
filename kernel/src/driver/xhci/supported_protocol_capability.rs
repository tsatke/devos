use bitfield::bitfield;

bitfield! {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct SupportedProtocolCapability(u128);
    impl Debug;

    pub u8, capability_id, _: 7, 0;
    pub u8, next_capability_pointer, _: 15, 8;
    pub u8, revision_minor, _: 23, 16;
    pub u8, revision_major, _: 31, 24;
    pub u32, name_bytes, _: 63, 32;
    pub u8, compatible_port_offset, _: 71, 64;
    pub u8, compatible_port_count, _: 79, 72;
    pub u16, protocol_defined, _: 91, 80;
    pub u8, psic, _: 95, 92;
    pub u8, protocol_slot_type, _: 100, 96;
}
