use crate::path::{AbsolutePath, Path, FILEPATH_SEPARATOR};
use alloc::string::String;
use core::borrow::Borrow;
use core::fmt::Display;
use core::ops::{Deref, DerefMut};
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

impl DerefMut for AbsoluteOwnedPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
#[error("path is not absolute")]
pub struct PathNotAbsoluteError;

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

impl Borrow<AbsolutePath> for AbsoluteOwnedPath {
    fn borrow(&self) -> &AbsolutePath {
        unsafe { AbsolutePath::new_unchecked(&self.inner) }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnedPath {
    inner: String,
}

impl Display for OwnedPath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl Deref for OwnedPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        Path::new(&self.inner)
    }
}

impl AsRef<Path> for OwnedPath {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl Borrow<Path> for OwnedPath {
    fn borrow(&self) -> &Path {
        self
    }
}

impl OwnedPath {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self { inner: s.into() }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Appends a string to the end of the path.
    ///
    /// ```rust
    /// # use vfs::path::OwnedPath;
    /// let mut path = OwnedPath::new("/foo");
    /// path.append_str(".txt");
    /// assert_eq!(path.as_str(), "/foo.txt");
    /// ```
    ///
    /// This is different from [`push_str`], which appends a string to
    /// the end of the path as a new component.
    pub fn append_str(&mut self, other: &str) {
        self.inner.push_str(other);
    }

    /// Appends a string to the end of the path as a new component.
    ///
    /// ```rust
    /// # use vfs::path::OwnedPath;
    /// let mut path = OwnedPath::new("/foo");
    /// path.push("bar");
    /// assert_eq!(path.as_str(), "/foo/bar");
    /// ```
    ///
    /// If the path is empty, pushing a new component will make
    /// the path absolute.
    /// ```rust
    /// # use vfs::path::OwnedPath;
    /// let mut path = OwnedPath::new("");
    /// path.push("foo");
    /// assert_eq!(path.as_str(), "/foo");
    /// ```
    pub fn push<P>(&mut self, other: P)
    where
        P: AsRef<Path>,
    {
        let other = other.as_ref();
        if self.inner.ends_with(FILEPATH_SEPARATOR) {
            self.inner.push_str(other);
        } else {
            self.inner.push(FILEPATH_SEPARATOR);
            self.inner.push_str(other);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_str() {
        let mut path = OwnedPath::new("");
        path.append_str("foo");
        assert_eq!(path.as_str(), "foo");
        path.append_str("bar");
        assert_eq!(path.as_str(), "foobar");
        path.append_str("/");
        assert_eq!(path.as_str(), "foobar/");
        path.append_str("baz");
        assert_eq!(path.as_str(), "foobar/baz");
        path.append_str(".txt");
        assert_eq!(path.as_str(), "foobar/baz.txt");
    }

    #[test]
    fn test_push() {
        let mut path = OwnedPath::new("");
        path.push("foo");
        assert_eq!(path.as_str(), "/foo");
        path.push("bar");
        assert_eq!(path.as_str(), "/foo/bar");
        path.push(".txt");
        assert_eq!(path.as_str(), "/foo/bar/.txt");
    }
}
