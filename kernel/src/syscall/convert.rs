use core::fmt::Pointer;
use core::ops::Deref;
use core::slice::{from_raw_parts, from_raw_parts_mut};

use derive_more::Display;
use x86_64::VirtAddr;

use kernel_api::syscall::Errno;

const USERSPACE_END: usize = 0x8000_0000_0000;

#[derive(Display, Debug, Copy, Clone, Eq, PartialEq)]
pub struct NotInUserspace;

impl core::error::Error for NotInUserspace {}

pub struct UserspaceAddress(VirtAddr);

impl Pointer for UserspaceAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:p}", self.0.as_ptr::<()>())
    }
}

impl TryFrom<usize> for UserspaceAddress {
    type Error = NotInUserspace;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value >= USERSPACE_END {
            // TODO: check whether this address is correct
            return Err(NotInUserspace);
        }

        Ok(Self(
            VirtAddr::try_new(value as u64).map_err(|_| NotInUserspace)?,
        ))
    }
}

impl Deref for UserspaceAddress {
    type Target = VirtAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct UserspaceRange {
    start: UserspaceAddress,
    len: usize,
}

impl UserspaceRange {
    pub fn try_from(start: UserspaceAddress, len: usize) -> Result<Self, NotInUserspace> {
        assert!(len > 0);
        let end_incl = *start + len - 1_usize;
        let _ = UserspaceAddress::try_from(end_incl.as_u64() as usize)?;

        Ok(Self { start, len })
    }
}

pub trait TryFromUserspaceAddress: Sized {
    type Error;

    fn try_from_userspace_addr(addr: UserspaceAddress) -> Result<Self, Self::Error>;
}

pub trait TryFromUserspaceRange: Sized {
    type Error;

    fn try_from_userspace_range(range: UserspaceRange) -> Result<Self, Self::Error>;
}

impl TryFromUserspaceAddress for &str {
    type Error = Errno;

    fn try_from_userspace_addr(addr: UserspaceAddress) -> Result<Self, Self::Error> {
        let ptr = addr.as_ptr();
        let len = strlen_s(ptr, 255).ok_or(Errno::ENAMETOOLONG)?;
        let slice = unsafe { from_raw_parts(ptr, len) };
        core::str::from_utf8(slice).map_err(|_| Errno::EINVAL)
    }
}

fn strlen_s(ptr: *const u8, max: usize) -> Option<usize> {
    (0..max).find(|&i| unsafe { *ptr.add(i) } == 0)
}

impl TryFromUserspaceRange for &mut [u8] {
    type Error = Errno;

    fn try_from_userspace_range(range: UserspaceRange) -> Result<Self, Self::Error> {
        let ptr = range.start.as_mut_ptr::<u8>();
        let len = range.len;
        Ok(unsafe { from_raw_parts_mut(ptr, len) })
    }
}

impl TryFromUserspaceRange for &[u8] {
    type Error = Errno;

    fn try_from_userspace_range(range: UserspaceRange) -> Result<Self, Self::Error> {
        let ptr = range.start.as_ptr::<u8>();
        let len = range.len;
        Ok(unsafe { from_raw_parts(ptr, len) })
    }
}
