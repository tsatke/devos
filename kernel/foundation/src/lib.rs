//! A collection of very kernel specific building blocks.
//!
//! This module contains foundational building blocks that are kernel-specific.
//! If the need arises, feel free to move stuff into a separate crate.

#![no_std]
#![feature(allocator_api)]
#![feature(box_into_inner)]
#![feature(extend_one_unchecked)]
#![feature(vec_push_within_capacity)]
extern crate alloc;

pub mod falloc;
pub mod future;
pub mod io;
pub mod mem;
