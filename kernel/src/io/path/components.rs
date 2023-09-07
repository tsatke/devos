use crate::io::path::{Path, SEPARATOR};
use core::iter::FusedIterator;

#[derive(Clone)]
pub struct Components<'a> {
    path: &'a str,
    state_front: State,
    state_back: State,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    StartDir,
    Body,
    Done,
}

impl<'a> Components<'a> {
    pub fn new(path: &'a Path) -> Self {
        Components {
            path: &path.inner,
            state_front: if path.starts_with(SEPARATOR) {
                State::StartDir
            } else {
                State::Body
            },
            state_back: State::Body,
        }
    }

    pub fn as_path(&self) -> &'a Path {
        self.path.into()
    }

    fn next_component_front(&mut self) -> (usize, Option<Component<'a>>) {
        debug_assert_eq!(State::Body, self.state_front);
        if self.path.is_empty() {
            return (0, None); // nothing more to parse
        }

        let (extra, component) = match self
            .path
            .as_bytes()
            .iter()
            .position(|&b| b as char == SEPARATOR)
        {
            None => (0, self.path),
            Some(i) => (1, &self.path[..i]),
        };
        (component.len() + extra, Some(component.into()))
    }

    fn next_component_back(&mut self) -> (usize, Option<Component<'a>>) {
        debug_assert_eq!(State::Body, self.state_back);
        let (extra, component) = match self
            .path
            .as_bytes()
            .iter()
            .rposition(|&b| b as char == SEPARATOR)
        {
            None => (0, self.path),
            Some(i) => (1, &self.path[i + 1..]),
        };
        (component.len() + extra, Some(component.into()))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Component<'a> {
    /// The root dir component. If it is encountered, it is always
    /// the first component of a path, and the path is absolute.
    /// If this is not encountered, then the path is not absolute.
    RootDir,
    /// The current directory, produced if the path contains a `.`
    /// as a part, such as in `/foo/./bar`.
    CurrentDir,
    /// The parent directory, produced if the path contains a `..`
    /// as a part, such as in `/foo/../bar`.
    ParentDir,
    /// A normal component, which is a non-empty string. Empty
    /// parts in the path are ignored, meaning that a path like
    /// `foo//bar` and `foo///bar` will have two components,
    /// `foo` and `bar`.
    Normal(&'a str),
}

impl<'a> From<&'a str> for Component<'a> {
    fn from(s: &'a str) -> Self {
        match s {
            "." => Component::CurrentDir,
            ".." => Component::ParentDir,
            "/" => Component::RootDir, // '/' as a component means root dir, everything else is considered a separator and must not reach this
            _ => Component::Normal(s),
        }
    }
}

impl<'a> FusedIterator for Components<'a> {}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: while !self.path.is_empty() {
            return match self.state_front {
                State::StartDir => {
                    self.state_front = State::Body;
                    self.path = &self.path[SEPARATOR.len_utf8()..];
                    Some(Component::RootDir)
                }
                State::Body => {
                    if let (count, Some(comp)) = self.next_component_front() {
                        self.path = &self.path[count..];
                        if let Component::Normal("") = comp {
                            continue 'outer; // don't return empty fragments
                        }
                        Some(comp)
                    } else {
                        self.state_front = State::Done;
                        None
                    }
                }
                State::Done => None,
            };
        }
        None
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        'outer: while !self.path.is_empty() {
            return match self.state_back {
                State::StartDir => {
                    self.state_back = State::Done;
                    Some(Component::RootDir)
                }
                State::Body => {
                    if let (count, Some(comp)) = self.next_component_back() {
                        self.path = &self.path[..self.path.len() - count];
                        if let Component::Normal("") = comp {
                            continue 'outer; // don't return empty fragments
                        }
                        Some(comp)
                    } else {
                        self.state_back = State::Done;
                        None
                    }
                }
                State::Done => None,
            };
        }
        None
    }
}
