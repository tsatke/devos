use crate::io::path::{Component, Components, Path, SEPARATOR};
use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::{Display, Formatter};

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub struct OwnedPath {
    inner: String,
}

impl<P: AsRef<Path>> From<P> for OwnedPath {
    fn from(v: P) -> Self {
        let mut s = Self::new();
        s.push(v);
        s
    }
}

impl Borrow<Path> for OwnedPath {
    fn borrow(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl Default for OwnedPath {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnedPath {
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.inner)
    }

    pub fn push<P: AsRef<Path>>(&mut self, segment: P) {
        let path = segment.as_ref();

        path.components().for_each(|c| {
            if !self.is_empty() && self.inner.chars().last().unwrap() != SEPARATOR {
                // we need to push a separator if the rightmost char is not a separator
                self.inner.push(SEPARATOR);
            }

            match c {
                Component::CurrentDir => self.inner.push('.'),
                Component::ParentDir => {
                    self.inner.push_str("..");
                }
                Component::Normal(s) => self.inner.push_str(s),
                Component::RootDir => {
                    if self.is_empty() {
                        self.inner.push(SEPARATOR);
                    }
                }
            }
        });
    }

    pub fn into_components(self) -> Vec<OwnedPath> {
        let mut data: Vec<OwnedPath> = Vec::new();

        self.components().for_each(|c| {
            match c {
                Component::ParentDir => {
                    while let Some(last) = data.last() {
                        if last.inner == "." {
                            data.pop();
                        } else {
                            break;
                        }
                    }
                    data.pop();
                }
                Component::CurrentDir => {}
                Component::RootDir => {}
                Component::Normal(p) => data.push(p.into()),
            };
        });

        data
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn components(&self) -> Components<'_> {
        Path::new(&self.inner).components()
    }
}

impl Display for OwnedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
