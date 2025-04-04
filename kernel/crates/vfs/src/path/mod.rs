use alloc::borrow::ToOwned;
use core::fmt::{Display, Formatter};
use core::ops::Deref;
use core::ptr;

pub use filenames::*;
pub use owned::*;

mod filenames;
mod owned;

pub const FILEPATH_SEPARATOR: char = '/';

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Path {
    inner: str,
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl AsRef<Path> for &str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<str> for &Path {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Path {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        unsafe { &*(ptr::from_ref::<str>(s.as_ref()) as *const Path) }
    }

    #[must_use]
    pub fn filenames(&self) -> Filenames<'_> {
        Filenames::new(self)
    }

    #[must_use]
    pub fn is_absolute(&self) -> bool {
        self.inner.starts_with(FILEPATH_SEPARATOR)
    }

    #[must_use]
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.filenames().last()
    }
}

impl ToOwned for Path {
    type Owned = OwnedPath;

    fn to_owned(&self) -> Self::Owned {
        Self::Owned::new(self.inner.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use crate::path::Path;

    #[test]
    fn test_file_name() {
        assert_eq!(Path::new("").file_name(), None);
        assert_eq!(Path::new("/").file_name(), None);
        assert_eq!(Path::new("//").file_name(), None);
        assert_eq!(Path::new("foo").file_name(), Some("foo"));
        assert_eq!(Path::new("/foo").file_name(), Some("foo"));
        assert_eq!(Path::new("//foo").file_name(), Some("foo"));
        assert_eq!(Path::new("foo/").file_name(), Some("foo"));
        assert_eq!(Path::new("/foo/").file_name(), Some("foo"));
        assert_eq!(Path::new("/foo//bar/").file_name(), Some("bar"));
    }

    #[test]
    fn test_is_absolute() {
        assert!(!Path::new("").is_absolute());

        assert!(Path::new("/").is_absolute());
        assert!(Path::new("//").is_absolute());
        assert!(Path::new("///").is_absolute());

        assert!(!Path::new(" ").is_absolute());
        assert!(!Path::new(" /").is_absolute());

        assert!(!Path::new("foo").is_absolute());
        assert!(Path::new("/foo/bar").is_absolute());
        assert!(!Path::new("foo/bar").is_absolute());
    }
}
