use crate::path::{Path, FILEPATH_SEPARATOR};
use core::str::CharIndices;

pub struct Filenames<'a> {
    inner: &'a Path,
    chars: CharIndices<'a>,
    index_front: usize,
    index_back: usize,
}

impl<'a> Filenames<'a> {
    #[must_use]
    pub fn new(p: &'a Path) -> Filenames<'a> {
        Self {
            inner: p,
            chars: p.inner.char_indices(),
            index_front: 0,
            index_back: p.inner.len(),
        }
    }
}

impl<'a> Iterator for Filenames<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.inner.is_empty() {
            return None;
        }

        self.chars.find(|(_, c)| c != &FILEPATH_SEPARATOR);
        self.index_front = self.chars.offset() - 1;

        let next_pos = self
            .chars
            .find(|(_, c)| c == &FILEPATH_SEPARATOR)
            .map(|v| v.0);
        if next_pos.is_some() || self.index_front < self.index_back.saturating_sub(1) {
            let filename = &self.inner.inner
                [self.index_front..self.chars.offset() - usize::from(next_pos.is_some())];
            self.index_front = self.chars.offset();
            Some(filename)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for Filenames<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.inner.inner.is_empty() {
            return None;
        }

        self.index_back = self
            .chars
            .rfind(|(_, c)| c != &FILEPATH_SEPARATOR)
            .map_or(0, |v| v.0 + 1);

        let prev_pos = self
            .chars
            .rfind(|(_, c)| c == &FILEPATH_SEPARATOR)
            .map_or(self.index_front, |v| v.0 + 1);
        if self.index_back > self.index_front {
            let filename = &self.inner.inner[prev_pos..self.index_back];
            self.index_back = prev_pos;
            Some(filename)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filenames_iter_front_back() {
        {
            let path = Path::new("/foo/bar/baz");
            let mut filenames = path.filenames();
            assert_eq!(filenames.next(), Some("foo"));
            assert_eq!(filenames.next(), Some("bar"));
            assert_eq!(filenames.next_back(), Some("baz"));
            assert_eq!(filenames.next(), None);
            assert_eq!(filenames.next_back(), None);
        }
        {
            let path = Path::new("/foo/bar/baz");
            let mut filenames = path.filenames();
            assert_eq!(filenames.next(), Some("foo"));
            assert_eq!(filenames.next_back(), Some("baz"));
            assert_eq!(filenames.next(), Some("bar"));
            assert_eq!(filenames.next_back(), None);
            assert_eq!(filenames.next(), None);
        }
    }

    #[test]
    fn test_filenames() {
        for (path, expected) in &[
            ("", &[][..]),
            ("/", &[]),
            ("//", &[]),
            ("///", &[]),
            (" /", &[" "]),
            ("foo", &["foo"]),
            ("/foo", &["foo"]),
            ("//foo", &["foo"]),
            ("foo/", &["foo"]),
            ("foo//", &["foo"]),
            ("/foo/", &["foo"]),
            ("//foo//", &["foo"]),
            ("foo/bar", &["foo", "bar"]),
            ("/foo/bar", &["foo", "bar"]),
            ("//foo/bar", &["foo", "bar"]),
            ("///foo/bar", &["foo", "bar"]),
            ("foo/bar/", &["foo", "bar"]),
            ("foo/bar//", &["foo", "bar"]),
            ("foo/bar///", &["foo", "bar"]),
            ("foo//bar", &["foo", "bar"]),
            ("foo///bar", &["foo", "bar"]),
            ("///foo///bar///", &["foo", "bar"]),
            ("/foo/bar/baz", &["foo", "bar", "baz"]),
            ("foo/bar/baz", &["foo", "bar", "baz"]),
            ("/foo/bar/baz/", &["foo", "bar", "baz"]),
            ("//foo/bar/baz/", &["foo", "bar", "baz"]),
            ("//foo/bar/baz//", &["foo", "bar", "baz"]),
            ("///foo/bar/baz//", &["foo", "bar", "baz"]),
            ("///foo/bar/baz///", &["foo", "bar", "baz"]),
        ] {
            let path = Path::new(path);
            // iterator
            {
                let mut filenames = path.filenames();
                for (i, expected) in expected.iter().enumerate() {
                    assert_eq!(
                        filenames.next(),
                        Some(*expected),
                        "at index {}, for path '{}'",
                        i,
                        path
                    );
                }
                assert_eq!(filenames.next(), None, "for path '{}'", path);
            }

            // double-ended iterator
            {
                let mut filenames = path.filenames();
                for (i, expected) in expected.iter().rev().enumerate() {
                    assert_eq!(
                        filenames.next_back(),
                        Some(*expected),
                        "at index {}, for path '{}'",
                        i,
                        path
                    );
                }
                assert_eq!(filenames.next_back(), None, "for path '{}'", path);
            }
        }
    }
}
