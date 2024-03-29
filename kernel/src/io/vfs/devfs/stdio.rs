use crate::io::vfs::devfs::DevFile;
use crate::io::vfs::error::Result;
use crate::io::vfs::VfsError;
use crate::process::fd::Fileno;
use crate::serial_print;

pub const STDIN: StdFile = StdFile(Fileno::new(0));
pub const STDOUT: StdFile = StdFile(Fileno::new(1));
pub const STDERR: StdFile = StdFile(Fileno::new(2));

#[derive(Debug, Eq, PartialEq)]
pub struct StdFile(Fileno);

impl DevFile for StdFile {
    fn read(&self, _: &mut [u8], _: usize) -> Result<usize> {
        unimplemented!()
    }

    fn write(&mut self, buf: &[u8], _: usize) -> Result<usize> {
        if self == &STDIN {
            return Err(VfsError::WriteError);
        }
        let text = core::str::from_utf8(buf).map_err(|_| VfsError::WriteError)?;
        serial_print!("{}", text);
        Ok(buf.len())
    }
}
