use crate::foundation::io::{Read, ReadError, ReadResult};
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
                Ok(ReadResult::Read(n)) => {
                    debug_assert_eq!(1, n.get());
                    return Some(Ok(byte));
                }
                Ok(ReadResult::TryAgain) => continue,
                Ok(ReadResult::EndOfStream) => return None,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
