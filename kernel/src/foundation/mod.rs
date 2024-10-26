//! A collection of very kernel specific building blocks.
//!
//! This module contains foundational building blocks that are kernel-specific.
//! If the need arises, feel free to move stuff into a separate crate.

pub mod falloc;
pub mod io;
pub mod mem;
