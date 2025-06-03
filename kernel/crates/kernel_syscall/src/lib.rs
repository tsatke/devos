#![no_std]
#![feature(negative_impls)]
extern crate alloc;

use core::ops::{Deref, DerefMut};
use core::ptr::{with_exposed_provenance, with_exposed_provenance_mut};
use kernel_abi::{EINVAL, Errno};
use thiserror::Error;

pub mod fcntl;

pub struct UserspacePtr<T> {
    ptr: *const T,
}

#[derive(Debug, Error)]
#[error("not a userspace pointer: 0x{0:#x}")]
pub struct NotUserspace(usize);

impl From<NotUserspace> for Errno {
    fn from(_: NotUserspace) -> Self {
        EINVAL
    }
}

impl<T> UserspacePtr<T> {
    /// # Safety
    /// The caller must ensure that the passed address is a valid pointer.
    /// It is explicitly safe to pass a pointer that is not in userspace.
    pub unsafe fn try_from_usize(ptr: usize) -> Result<Self, NotUserspace> {
        #[cfg(not(target_pointer_width = "64"))]
        compile_error!("only 64bit pointer width is supported");
        if ptr & 1 << 63 != 0 {
            Err(NotUserspace(ptr))
        } else {
            Ok(Self {
                ptr: with_exposed_provenance(ptr),
            })
        }
    }

    /// # Safety
    /// The caller must ensure that the pointer is valid and points to a slice of `len` elements.
    pub unsafe fn as_slice(&self, len: usize) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr, len) }
    }
}

impl<T> Deref for UserspacePtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY: The pointer is valid, which is an invariant of this type.
            &*self.ptr
        }
    }
}

pub struct UserspaceMutPtr<T> {
    ptr: *mut T,
}

impl<T> !Clone for UserspaceMutPtr<T> {}

impl<T> UserspaceMutPtr<T> {
    /// # Safety
    /// The caller must ensure that the passed address is a valid mutable pointer.
    /// It is explicitly safe to pass a pointer that is not in userspace.
    pub unsafe fn try_from_usize(ptr: usize) -> Result<Self, NotUserspace> {
        #[cfg(not(target_pointer_width = "64"))]
        compile_error!("only 64bit pointer width is supported");
        if ptr & 1 << 63 != 0 {
            Err(NotUserspace(ptr))
        } else {
            Ok(Self {
                ptr: with_exposed_provenance_mut(ptr),
            })
        }
    }
}

impl<T> Deref for UserspaceMutPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY: The pointer is valid, which is an invariant of this type.
            &*self.ptr
        }
    }
}

impl<T> DerefMut for UserspaceMutPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // SAFETY: The pointer is valid for mutable, which is an invariant of this type.
            &mut *self.ptr
        }
    }
}
