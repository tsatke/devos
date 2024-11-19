use foundation::io::{Write, WriteExactError};

pub trait WireSerializable<T: Write<u8>> {
    fn serialize(&self, s: &mut WireSerializer<T>) -> Result<(), WriteExactError>;
}

impl<T, U> WireSerializable<T> for &'_ U
where
    U: WireSerializable<T>,
    T: Write<u8>,
{
    fn serialize(&self, s: &mut WireSerializer<T>) -> Result<(), WriteExactError> {
        s.write_serializable(*self)
    }
}

pub struct WireSerializer<T> {
    inner: T,
}

impl<T> WireSerializer<T>
where
    T: Write<u8>,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), WriteExactError> {
        self.inner.write_exact(&[value])
    }

    pub fn write_u16(&mut self, value: u16) -> Result<(), WriteExactError> {
        self.inner.write_exact(&value.to_be_bytes())
    }

    pub fn write_raw(&mut self, value: impl AsRef<[u8]>) -> Result<(), WriteExactError> {
        self.inner.write_exact(value.as_ref())
    }

    pub fn write_from(&mut self, value: impl Iterator<Item = u8>) -> Result<(), WriteExactError> {
        for byte in value {
            self.write_u8(byte)?;
        }
        Ok(())
    }

    pub fn write_serializable(
        &mut self,
        value: impl WireSerializable<T>,
    ) -> Result<(), WriteExactError> {
        value.serialize(self)
    }
}
