pub trait Device {}

pub trait BlockDevice: Device {
    type Error;

    fn block_size(&self) -> usize;

    fn read_block(&mut self, block: usize, buf: &mut [u8]) -> Result<(), Self::Error>;

    fn read_multiple_blocks(
        &mut self,
        start_block: usize,
        block_count: usize,
        buf: &mut [u8],
    ) -> Result<(), Self::Error> {
        let block_size = self.block_size();

        for (i, block) in (start_block..start_block + block_count).enumerate() {
            self.read_block(block, &mut buf[i * block_size..(i + 1) * block_size])?;
        }

        Ok(())
    }

    fn read_into(&mut self, start_block: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
        let block_size = self.block_size();
        let full_block_count = buf.len().div_floor(block_size);
        self.read_multiple_blocks(start_block, full_block_count, buf)?;
        if full_block_count != buf.len() {
            // the slice can hold an incomplete block, but we can only read full blocks
            let mut block_buf = [0_u8; 512];
            self.read_block(start_block + full_block_count, &mut block_buf)?;
            let remaining_buf = &mut buf[full_block_count * block_size..];
            remaining_buf.copy_from_slice(&block_buf[..remaining_buf.len()])
        }
        Ok(())
    }

    fn write_block(&mut self, block: usize, buf: &[u8]) -> Result<(), Self::Error>;

    fn write_multiple_blocks(
        &mut self,
        start_block: usize,
        block_count: usize,
        buf: &[u8],
    ) -> Result<(), Self::Error> {
        let block_size = self.block_size();

        for (i, block) in (start_block..start_block + block_count).enumerate() {
            self.write_block(block, &buf[i * block_size..(i + 1) * block_size])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    struct TestDevice(Vec<u8>);

    impl Default for TestDevice {
        fn default() -> Self {
            let mut inner = vec![0_u8; 512 * 512]; // 256k
            for block in 0..512 {
                for offset in 0..512 {
                    inner[(block * 512) + offset] = block as u8; // intended overflow
                }
            }
            Self(inner)
        }
    }

    impl Device for TestDevice {}

    impl BlockDevice for TestDevice {
        type Error = ();

        fn block_size(&self) -> usize {
            512
        }

        fn read_block(&mut self, block: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
            if buf.len() < self.block_size() {
                return Err(());
            }

            buf.copy_from_slice(
                &self.0[block * self.block_size()..(block + 1) * self.block_size()],
            );
            Ok(())
        }

        fn write_block(&mut self, block: usize, buf: &[u8]) -> Result<(), Self::Error> {
            if buf.len() < self.block_size() {
                return Err(());
            }

            let block_size = self.block_size();
            self.0[block * block_size..(block + 1) * block_size].copy_from_slice(buf);
            Ok(())
        }
    }

    #[test]
    fn _test_device() {
        // ensure that TestDevice works in the way that it's intended

        let dev = TestDevice::default();
        assert_eq!(512, dev.block_size());
    }

    #[test]
    fn test_read_multiple_blocks() {
        let mut dev = TestDevice::default();
        let mut buf = vec![0_u8; dev.block_size() * 4];
        dev.read_multiple_blocks(12, 3, &mut buf).unwrap();
        buf[0..512].iter().for_each(|&e| assert_eq!(12, e));
        buf[512..1024].iter().for_each(|&e| assert_eq!(13, e));
        buf[1024..1536].iter().for_each(|&e| assert_eq!(14, e));
        buf[1536..2048].iter().for_each(|&e| assert_eq!(0, e));
    }

    #[should_panic]
    #[test]
    fn test_read_multiple_blocks_too_short() {
        let mut dev = TestDevice::default();
        let mut buf = vec![0_u8; 512];
        let _ = dev.read_multiple_blocks(1, 2, &mut buf);
    }

    #[test]
    fn test_write_multiple_blocks() {
        let mut dev = TestDevice::default();
        let write_buf: Vec<_> = (0..dev.block_size() * 2).map(|v| v as u8).collect();
        dev.write_multiple_blocks(13, 2, &write_buf).unwrap();
        let mut read_buf = vec![0_u8; dev.block_size() * 4];
        dev.read_multiple_blocks(12, 4, &mut read_buf).unwrap();
        read_buf[0..512].iter().for_each(|&e| assert_eq!(12, e));
        read_buf[512..1536]
            .iter()
            .enumerate()
            .for_each(|(i, &e)| assert_eq!(i as u8, e));
        read_buf[1536..2048].iter().for_each(|&e| assert_eq!(15, e));
    }
}
