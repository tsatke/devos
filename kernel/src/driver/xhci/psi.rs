use bitfield::bitfield;

bitfield! {
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct Psi(u32);
    impl Debug;

    pub u8, psiv, _: 3, 0;
    pub u8, psie, _: 5, 4;
    pub u8, plt, _: 7, 6;
    pub bool, pfd, _: 8;
    pub u8, lp, _: 15, 14;
    pub u16, psim, _: 31, 16;
}
