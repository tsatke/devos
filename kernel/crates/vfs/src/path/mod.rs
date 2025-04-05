use alloc::borrow::{Cow, ToOwned};
use core::fmt::{Display, Formatter};
use core::ops::Deref;
use core::ptr;
pub use filenames::*;
pub use owned::*;
use thiserror::Error;

mod filenames;
mod owned;

pub const FILEPATH_SEPARATOR: char = '/';

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct AbsolutePath {
    inner: Path,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
#[error("path is not absolute")]
pub struct PathNotAbsoluteError;

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

impl AsRef<Path> for &Path {
    fn as_ref(&self) -> &Path {
        self
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
        self.starts_with(FILEPATH_SEPARATOR)
    }

    #[must_use]
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.filenames().next_back()
    }

    #[must_use]
    pub fn parent(&self) -> Option<&Path> {
        let mut chars = self.char_indices();
        chars.rfind(|&(_, c)| c != FILEPATH_SEPARATOR);
        chars.rfind(|&(_, c)| c == FILEPATH_SEPARATOR);
        chars
            .rfind(|&(_, c)| c != FILEPATH_SEPARATOR)
            .map(|v| v.0 + 1)
            .map(|offset| Path::new(&self.inner[..offset]))
    }

    #[must_use]
    pub fn make_absolute(&self) -> Cow<Self> {
        if self.is_absolute() {
            Cow::Borrowed(self)
        } else {
            let mut p = OwnedPath::new("/");
            p.push(self);
            Cow::Owned(p)
        }
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
    use crate::path::{OwnedPath, Path};
    use alloc::borrow::Cow;

    #[test]
    fn test_make_absolute() {
        for (path, expected) in [
            ("", Cow::Owned(OwnedPath::new("/"))),
            ("/", Cow::Borrowed(Path::new("/"))),
            ("//", Cow::Borrowed(Path::new("//"))),
            ("foo", Cow::Owned(OwnedPath::new("/foo"))),
            ("/foo", Cow::Borrowed(Path::new("/foo"))),
            ("foo/bar", Cow::Owned(OwnedPath::new("/foo/bar"))),
            ("/foo/bar", Cow::Borrowed(Path::new("/foo/bar"))),
            ("//foo/bar", Cow::Borrowed(Path::new("//foo/bar"))),
            ("///foo/bar", Cow::Borrowed(Path::new("///foo/bar"))),
        ] {
            assert_eq!(Path::new(path).make_absolute(), expected);
        }
    }

    #[test]
    fn test_parent() {
        for (path, parent) in [
            ("/foo/bar/baz", Some("/foo/bar")),
            ("/foo/bar", Some("/foo")),
            ("/foo//bar", Some("/foo")),
            ("///foo/bar", Some("///foo")),
            ("foo", None),
            ("/foo", None),
            ("//foo", None),
            ("foo/", None),
            ("/foo/", None),
            ("/foo/bar/baz/", Some("/foo/bar")),
            ("/foo/bar/baz//", Some("/foo/bar")),
            ("/foo/bar/baz///", Some("/foo/bar")),
            ("/foo/bar//baz///", Some("/foo/bar")),
            ("/foo/bar///baz///", Some("/foo/bar")),
            ("///foo///bar///baz///", Some("///foo///bar")),
        ] {
            assert_eq!(Path::new(path).parent(), parent.map(Path::new));
        }
    }

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
