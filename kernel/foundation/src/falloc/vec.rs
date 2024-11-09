use crate::io::{Write, WriteError};
use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::fmt::Debug;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::slice::SliceIndex;
use delegate::delegate;

pub struct FVec<T> {
    inner: Vec<T>,
}

impl<T: Debug> Debug for FVec<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl<T> From<Vec<T>> for FVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> AsRef<[T]> for FVec<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for FVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.inner
    }
}

impl<T> Deref for FVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl<T> DerefMut for FVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut()
    }
}

impl<T> Borrow<[T]> for FVec<T> {
    fn borrow(&self) -> &[T] {
        &self[..]
    }
}

impl<T> BorrowMut<[T]> for FVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self[..]
    }
}

impl<T, I> Index<I> for FVec<T>
where
    I: SliceIndex<[T]>,
{
    type Output = <Vec<T> as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T, I> IndexMut<I> for FVec<T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<T> FVec<T>
where
    T: Default + Clone,
{
    pub fn try_with_len(len: usize) -> Result<FVec<T>, TryReserveError> {
        let mut v = Vec::new();
        v.try_reserve(len)?;
        v.resize(len, T::default());
        Ok(v.into())
    }
}

impl<T> FVec<T> {
    delegate! {
        to Vec {
            #[into]
            pub fn new() -> Self;
        }

        to self.inner {
            pub fn clear(&mut self);
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
            pub fn pop(&mut self) -> Option<T>;
            pub fn push_within_capacity(&mut self, t: T) -> Result<(), T>;
            pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError>;
            pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError>;
        }
    }

    pub fn try_with_capacity(capacity: usize) -> Result<FVec<T>, TryReserveError> {
        let mut r = Self::new();
        r.try_reserve(capacity)?;
        Ok(r)
    }

    pub fn try_push(&mut self, t: T) -> Result<(), T> {
        if self.try_reserve(1).is_err() {
            return Err(t);
        }
        self.push_within_capacity(t)
    }

    pub fn try_extend<I: IntoIterator<Item = T>>(
        &mut self,
        iter: I,
    ) -> Result<(), TryReserveError> {
        // TODO: once https://github.com/rust-lang/rust/issues/31844 (specialization) is stabilized, we can specialize try_extend for ExactSizeIterator
        for t in iter {
            unsafe {
                // call directly on inner to make sure we can't mess things up by
                // relying on that impl in extend_one_unchecked below
                self.inner.try_reserve(1)?;
                // Safety: we just called try_reserve for an additional element
                self.inner.extend_one_unchecked(t);
            }
        }
        Ok(())
    }

    pub fn try_resize_with<F>(&mut self, new_len: usize, f: F) -> Result<(), TryReserveError>
    where
        F: FnMut() -> T,
    {
        if new_len > self.len() {
            self.try_reserve_exact(new_len - self.len())?;
        }
        self.inner.resize_with(new_len, f); // will not allocate
        Ok(())
    }
}

impl<T> Write<T> for FVec<T>
where
    T: Clone,
{
    fn write(&mut self, buf: &[T]) -> Result<usize, WriteError> {
        if buf.len() == 0 {
            return Ok(0);
        }

        for t in buf {
            self.try_push(t.clone())
                .map_err(|_| WriteError::EndOfStream)?;
        }
        Ok(buf.len())
    }
}
