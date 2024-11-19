use foundation::io::{Write, WriteError};

#[derive(Default)]
pub struct Checksum {
    scratch: Option<u8>,
    state: u32,
}

impl Checksum {
    pub fn write_byte(&mut self, byte: u8) {
        if let Some(scratch) = self.scratch {
            let v = u32::from(scratch) << 8 | u32::from(byte);
            self.state += v;
            self.scratch = None;
        } else {
            self.scratch = Some(byte);
        }
    }

    pub fn finalize(self) -> u16 {
        let carry = (self.state >> 16) as u16;
        let value = self.state as u16;
        !(value + carry)
    }
}

impl Write<u8> for Checksum {
    fn write(&mut self, buf: &[u8]) -> Result<usize, WriteError> {
        // TODO: this can probably be optimized with array_chunks, we should measure this first though
        buf.iter().for_each(|&byte| self.write_byte(byte));
        Ok(buf.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_checksum() {
        for (data, checksum) in [
            (
                vec![
                    0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0xb8, 0x61, 0xc0,
                    0xa8, 0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7,
                ],
                0x0000,
            ),
            (
                vec![
                    0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0x00, 0x00, 0xc0,
                    0xa8, 0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7,
                ],
                0xb861,
            ),
        ] {
            let expected: u16 = checksum;

            let mut comp = Checksum::default();
            comp.write_exact(&data).unwrap();

            let actual = comp.finalize();
            assert_eq!(
                expected, actual,
                "expected checksum {:#02X}, but got {:#02X}",
                expected, actual,
            );
        }
    }
}
