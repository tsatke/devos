use alloc::borrow::{Cow, ToOwned};
use core::fmt::{Display, Formatter};
use core::ops::Deref;
use core::ptr;
pub use filenames::*;
pub use owned::*;

mod filenames;
mod owned;

pub const FILEPATH_SEPARATOR: char = '/';

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct AbsolutePath {
    inner: Path,
}

impl AbsolutePath {
    pub const ROOT: &'static Self = unsafe { &*(ptr::from_ref::<str>("/") as *const AbsolutePath) };

    /// Creates a new [`AbsolutePath`] from a string slice.
    ///
    /// # Errors
    /// Returns an error if the path is not absolute.
    pub fn try_new(path: &str) -> Result<&Self, PathNotAbsoluteError> {
        path.try_into()
    }

    unsafe fn new_unchecked(path: &Path) -> &Self {
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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
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
    pub fn make_absolute(&self) -> Cow<AbsolutePath> {
        if let Ok(path) = AbsolutePath::try_new(self) {
            Cow::Borrowed(path)
        } else {
            let mut p = AbsoluteOwnedPath::new();
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
    use crate::path::Path;
    use alloc::borrow::Cow;

    #[test]
    fn test_make_absolute() {
        for (path, expected) in [
            ("", Cow::Owned("/".try_into().unwrap())),
            ("/", Cow::Borrowed("/".try_into().unwrap())),
            ("//", Cow::Borrowed("//".try_into().unwrap())),
            ("foo", Cow::Owned("/foo".try_into().unwrap())),
            ("/foo", Cow::Borrowed("/foo".try_into().unwrap())),
            ("foo/bar", Cow::Owned("/foo/bar".try_into().unwrap())),
            ("/foo/bar", Cow::Borrowed("/foo/bar".try_into().unwrap())),
            ("//foo/bar", Cow::Borrowed("//foo/bar".try_into().unwrap())),
            (
                "///foo/bar",
                Cow::Borrowed("///foo/bar".try_into().unwrap()),
            ),
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
