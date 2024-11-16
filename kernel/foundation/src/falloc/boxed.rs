use alloc::boxed::Box;
use core::alloc::AllocError;
use core::ops::Deref;
use core::pin::Pin;

pub struct FBox<T> {
    inner: Box<T>,
}

impl<T> Deref for FBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> From<Box<T>> for FBox<T> {
    fn from(value: Box<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> FBox<T> {
    pub fn try_new(v: T) -> Result<Self, AllocError> {
        Box::try_new(v).map(Into::into)
    }

    pub fn into_pin(self) -> Pin<Self> {
        unsafe { Pin::new_unchecked(self) }
    }

    pub fn into_raw(self) -> *mut T {
        Box::into_raw(self.inner)
    }

    /// # Safety
    /// The caller must ensure that the pointer was created by [`FBox::into_raw`].
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        unsafe { Box::from_raw(ptr) }.into()
    }

    pub fn into_inner(b: Self) -> T {
        Box::into_inner(b.inner)
    }
}
