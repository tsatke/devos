use alloc::string::String;
use core::borrow::Borrow;
use core::fmt::Display;
use core::ops::Deref;

use thiserror::Error;

use crate::path::{FILEPATH_SEPARATOR, Path};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
#[error("path is not absolute")]
pub struct PathNotAbsoluteError;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    /// # use kernel_vfs::path::OwnedPath;
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
    /// # use kernel_vfs::path::OwnedPath;
    /// let mut path = OwnedPath::new("/foo");
    /// path.push("bar");
    /// assert_eq!(path.as_str(), "/foo/bar");
    /// ```
    ///
    /// If the path is empty, pushing a new component will make
    /// the path absolute.
    /// ```rust
    /// # use kernel_vfs::path::OwnedPath;
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
