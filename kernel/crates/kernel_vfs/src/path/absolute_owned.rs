use crate::path::{AbsolutePath, OwnedPath, PathNotAbsoluteError};
use core::borrow::Borrow;
use core::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct AbsoluteOwnedPath {
    inner: OwnedPath,
}

impl Default for AbsoluteOwnedPath {
    fn default() -> Self {
        Self::new()
    }
}

impl AbsoluteOwnedPath {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: OwnedPath::new("/"),
        }
    }

    pub(crate) unsafe fn new_unchecked(inner: OwnedPath) -> Self {
        Self { inner }
    }
}

impl TryFrom<&str> for AbsoluteOwnedPath {
    type Error = PathNotAbsoluteError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let path = OwnedPath::new(value);
        path.try_into()
    }
}

impl Deref for AbsoluteOwnedPath {
    type Target = OwnedPath;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Borrow<AbsolutePath> for AbsoluteOwnedPath {
    fn borrow(&self) -> &AbsolutePath {
        unsafe { AbsolutePath::new_unchecked(&self.inner) }
    }
}

impl DerefMut for AbsoluteOwnedPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl TryFrom<OwnedPath> for AbsoluteOwnedPath {
    type Error = PathNotAbsoluteError;

    fn try_from(value: OwnedPath) -> Result<Self, Self::Error> {
        if value.is_absolute() {
            Ok(AbsoluteOwnedPath { inner: value })
        } else {
            Err(PathNotAbsoluteError)
        }
    }
}

impl AsRef<AbsolutePath> for AbsoluteOwnedPath {
    fn as_ref(&self) -> &AbsolutePath {
        unsafe { AbsolutePath::new_unchecked(&self.inner) }
    }
}
