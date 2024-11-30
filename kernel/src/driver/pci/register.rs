use crate::driver::pci::raw::{
    read_config_double_word, read_config_half_word, read_config_word, write_config_double_word,
    write_config_half_word, write_config_word,
};
use derive_more::{Deref, DerefMut, From};

macro_rules! impl_reg_ops {
    ($typ:ty, $read_fn:ident, $write_fn:ident) => {
        impl PciRegisterOps for $typ {
            fn read_pci_register(bus: u8, slot: u8, function: u8, offset: u8) -> Self {
                unsafe { $read_fn(bus, slot, function, offset) }
            }

            fn write_pci_register(bus: u8, slot: u8, function: u8, offset: u8, value: Self) {
                unsafe { $write_fn(bus, slot, function, offset, value) }
            }
        }
    };
}

impl_reg_ops!(u8, read_config_half_word, write_config_half_word);
impl_reg_ops!(u16, read_config_word, write_config_word);
impl_reg_ops!(u32, read_config_double_word, write_config_double_word);

#[derive(Debug, From, Deref, DerefMut)]
pub struct BaseAddressRegister(PciRegister<u32>);

impl BaseAddressRegister {
    /// Returns the lowest 3 bits of the bar. Bit0 is set if the bar is an I/O space bar.
    /// Bit2 is set if the bar is a 64bit bar. Bit1 should never be set.
    pub fn bar_type(&self) -> u8 {
        (self.0.read() & 0b111) as u8
    }

    pub fn prefetchable(&self) -> bool {
        (self.0.read() & 0b1000) > 0
    }

    pub fn is_mem_64bit(&self) -> bool {
        self.bar_type() == 0b100
    }

    pub fn is_mem_32bit(&self) -> bool {
        self.bar_type() == 0
    }

    pub fn is_io(&self) -> bool {
        self.bar_type() == 1
    }

    pub fn addr(&self, next: Option<&BaseAddressRegister>) -> usize {
        let v = self.0.read();
        if self.is_io() {
            (v & 0xFFFF_FFFC) as usize
        } else {
            (v & 0xFFFF_FFF0) as usize
                + if self.is_mem_64bit() {
                    (next.unwrap().read() as usize) << 32
                } else {
                    0
                }
        }
    }

    pub fn size(&mut self) -> usize {
        let original = self.0.read();
        self.0.write(!0);
        let v = self.0.read();
        self.0.write(original);
        !(v & if self.is_io() { !0b11 } else { !0b111 }) as usize + 1
    }

    pub fn exists(&self) -> bool {
        self.0.read() != 0
    }
}

#[derive(Debug)]
pub struct PciRegister<T> {
    bus: u8,
    slot: u8,
    function: u8,
    offset: u8,
    _output_type: core::marker::PhantomData<T>,
}

impl<T> PciRegister<T> {
    pub fn new(bus: u8, slot: u8, function: u8, offset: u8) -> Self {
        Self {
            bus,
            slot,
            function,
            offset,
            _output_type: core::marker::PhantomData,
        }
    }
}

pub trait PciRegisterOps {
    fn read_pci_register(bus: u8, slot: u8, function: u8, offset: u8) -> Self;
    fn write_pci_register(bus: u8, slot: u8, function: u8, offset: u8, value: Self);
}

impl<T> PciRegister<T>
where
    T: PciRegisterOps,
{
    pub fn read(&self) -> T {
        T::read_pci_register(self.bus, self.slot, self.function, self.offset)
    }

    pub fn write(&mut self, value: T) {
        T::write_pci_register(self.bus, self.slot, self.function, self.offset, value)
    }

    pub fn update(&mut self, f: impl FnOnce(T) -> T) {
        let value = self.read();
        self.write(f(value));
    }
}
