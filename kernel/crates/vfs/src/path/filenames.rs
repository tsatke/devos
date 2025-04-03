use crate::path::{Path, FILEPATH_SEPARATOR};
use core::str::CharIndices;

pub struct Filenames<'a> {
    inner: &'a Path,
    chars: CharIndices<'a>,
    index: usize,
}

impl<'a> Filenames<'a> {
    #[must_use]
    pub fn new(p: &'a Path) -> Filenames<'a> {
        Self {
            inner: p,
            chars: p.inner.char_indices(),
            index: 0,
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
        self.index = self.chars.offset() - 1;

        let next_pos = self
            .chars
            .find(|(_, c)| c == &FILEPATH_SEPARATOR)
            .map(|v| v.0);
        if next_pos.is_some() || self.index < self.inner.inner.len() - 1 {
            let filename = &self.inner.inner
                [self.index..self.chars.offset() - usize::from(next_pos.is_some())];
            self.index = self.chars.offset();
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
            ("/foo/bar/baz/", &["foo", "bar", "baz"]),
            ("//foo/bar/baz/", &["foo", "bar", "baz"]),
            ("//foo/bar/baz//", &["foo", "bar", "baz"]),
            ("///foo/bar/baz//", &["foo", "bar", "baz"]),
            ("///foo/bar/baz///", &["foo", "bar", "baz"]),
        ] {
            let path = Path::new(path);
            let mut filenames = path.filenames();
            for (i, expected) in expected.iter().enumerate() {
                assert_eq!(
                    filenames.next(),
                    Some(*expected),
                    "at index {}, for path {}",
                    i,
                    path
                );
            }
            assert_eq!(filenames.next(), None);
        }
    }
}
