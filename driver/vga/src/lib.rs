#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

pub use color::*;
#[allow(unused_imports)]
pub use drawing::*;
use graphics::{GraphicsDriver, Vec2};

mod color;
mod drawing;

pub(crate) const WIDTH: usize = 1280;
pub(crate) const HEIGHT: usize = 800;
pub(crate) type PixelType = u32;
pub(crate) const PIXEL_COUNT: usize = WIDTH * HEIGHT;

pub struct FrameBuffer(Vec<PixelType>);

impl FrameBuffer {
    /// # Safety
    /// The caller must ensure that this is a pointer to a frame buffer that is
    /// fully mapped in memory. This function will panic if that's not the case
    /// if `debug_assertions` are enabled.
    pub unsafe fn from_ptr(ptr: *mut PixelType) -> Self {
        let vec = Vec::from_raw_parts(ptr, PIXEL_COUNT, PIXEL_COUNT);
        #[cfg(debug_assertions)]
        core::ptr::read_volatile(vec.last().unwrap() as *const u32); // try to dereference the last element to make sure this doesn't page fault if the buffer is not fully mapped
        Self(vec)
    }
}

impl Deref for FrameBuffer {
    type Target = Vec<PixelType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrameBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Vga1280x800 {
    frame_buffer: FrameBuffer,
}

impl Vga1280x800 {
    pub fn new(frame_buffer: FrameBuffer) -> Self {
        Self { frame_buffer }
    }

    #[inline(always)]
    pub(crate) fn compute_frame_buffer_offset(&self, point: Vec2) -> usize {
        point.y * WIDTH + point.x
    }
}

impl GraphicsDriver for Vga1280x800 {
    fn init(&mut self) {
        // no-op
    }
}
