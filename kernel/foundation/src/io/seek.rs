use derive_more::Display;
use thiserror::Error;

pub trait Seek {
    /// Repositions the read/write index.
    ///
    /// Implementations may allow a seek beyond their current end-of-data.
    /// Seeks before the start-of-data are not allowed.
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, SeekError>;

    /// Returns the current read/write index.
    fn pos(&mut self) -> Result<usize, SeekError> {
        self.seek(SeekFrom::Current(0))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum SeekError {
    #[error("seek out of bounds")]
    SeekOutOfBounds,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum SeekFrom {
    Start(usize),
    End(isize),
    Current(isize),
}

/// For something that implements [`Seek`],
/// * save the current position
/// * seek to `offset`
/// * perform the function
/// * restore the original position
/// * return the result of the function
pub fn seek_do_restore<S, B, F, R, E1, E2>(s: &mut S, buf: B, offset: usize, f: F) -> Result<R, E2>
where
    F: FnOnce(&mut S, B) -> Result<R, E1>,
    S: Seek,
    E2: From<E1>,
    E2: From<SeekError>,
{
    let original_pos = s.pos()?;
    s.seek(SeekFrom::Start(offset))?;
    let res = f(s, buf);
    s.seek(SeekFrom::Start(original_pos))?;
    res.map_err(Into::into)
}
