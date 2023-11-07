#![no_std]

mod drawing;
mod vec2;

pub use drawing::*;
pub use vec2::*;

extern crate alloc;

pub trait GraphicsDriver {
    fn init(&mut self);
}

pub trait Color: Copy {}
