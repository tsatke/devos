use crate::io::path::{Component, Components, OwnedPath};
use alloc::borrow::ToOwned;
use alloc::string::String;
use core::fmt::{Display, Formatter};
use core::ops::Deref;

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Path {
    pub(in crate::io::path) inner: str,
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.inner)
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
        unsafe { &*(s.as_ref() as *const str as *const Path) }
    }

    pub fn components(&self) -> Components<'_> {
        Components::new(self)
    }

    pub fn parent(&self) -> Option<&Path> {
        let mut components = self.components();
        let last = components.next_back();
        last.and_then(|p| match p {
            Component::CurrentDir | Component::ParentDir | Component::Normal(_) => {
                let path = components.as_path();
                if path.is_empty() {
                    None
                } else {
                    Some(path)
                }
            }
            _ => None,
        })
    }
}

impl<'a> From<&'a str> for &'a Path {
    fn from(v: &'a str) -> Self {
        Path::new(v)
    }
}

impl ToOwned for Path {
    type Owned = OwnedPath;

    fn to_owned(&self) -> Self::Owned {
        self.into()
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}
