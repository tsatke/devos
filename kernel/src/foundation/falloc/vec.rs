use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;
use delegate::delegate;

pub struct FVec<T> {
    inner: Vec<T>,
}

impl Debug for FVec<u8> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl<T> From<Vec<T>> for FVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self { inner: value }
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
