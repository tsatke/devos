use crate::mem::virt::VmmError;
use core::error::Error;
use derive_more::Display;

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq)]
pub enum XhciError {
    NotUsb,
    VmmError(VmmError),
}

impl Error for XhciError {}
