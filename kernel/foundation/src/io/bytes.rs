use crate::io::{Read, ReadError};
use core::slice;

pub struct Bytes<R> {
    reader: R,
}

impl<R: Read<u8>> Bytes<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: Read<u8>> Iterator for Bytes<R> {
    type Item = Result<u8, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut byte = 0;
        loop {
            match self.reader.read(slice::from_mut(&mut byte)) {
                Ok(n) => {
                    debug_assert_eq!(1, n);
                    return Some(Ok(byte));
                }
                Err(ReadError::TryAgain) => continue,
                Err(ReadError::EndOfStream) => return None,
            }
        }
    }
}
