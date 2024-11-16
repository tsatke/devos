use crate::driver::usb::speed::Speed;

pub struct Device {
    port: usize,
    speed: Speed,
}

impl Device {
    pub fn port(&self) -> usize {
        self.port
    }

    pub fn speed(&self) -> Speed {
        self.speed
    }
}
