use bitfield::bitfield;

bitfield! {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct ExtendedCapabilities(u32);
    impl Debug;

    pub u8, id, _: 7, 0;
    pub u8, next_raw, _: 15, 8;
    pub u16, capability_specific, _: 31, 16;
}