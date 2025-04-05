use crate::path::{AbsoluteOwnedPath, Path, PathNotAbsoluteError};
use alloc::borrow::ToOwned;
use core::fmt::{Display, Formatter};
use core::ops::Deref;
use core::ptr;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct AbsolutePath {
    inner: Path,
}

impl Display for AbsolutePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl AbsolutePath {
    /// Creates a new [`AbsolutePath`] from a string slice.
    ///
    /// # Errors
    /// Returns an error if the path is not absolute.
    pub fn try_new(path: &str) -> Result<&Self, PathNotAbsoluteError> {
        path.try_into()
    }

    pub(crate) unsafe fn new_unchecked(path: &Path) -> &Self {
        unsafe { &*(ptr::from_ref::<Path>(path) as *const AbsolutePath) }
    }

    #[must_use]
    pub fn parent(&self) -> Option<&AbsolutePath> {
        self.inner
            .parent()
            .map(|v| unsafe { AbsolutePath::new_unchecked(v) })
    }
}

impl Deref for AbsolutePath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<AbsolutePath> for AbsolutePath {
    fn as_ref(&self) -> &AbsolutePath {
        self
    }
}

impl TryFrom<&str> for &AbsolutePath {
    type Error = PathNotAbsoluteError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Path::new(value).try_into()
    }
}

impl TryFrom<&Path> for &AbsolutePath {
    type Error = PathNotAbsoluteError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if value.is_absolute() {
            Ok(unsafe { &*(ptr::from_ref::<Path>(value) as *const AbsolutePath) })
        } else {
            Err(PathNotAbsoluteError)
        }
    }
}

impl ToOwned for AbsolutePath {
    type Owned = AbsoluteOwnedPath;

    fn to_owned(&self) -> Self::Owned {
        unsafe { AbsoluteOwnedPath::new_unchecked(self.inner.to_owned()) }
    }
}
