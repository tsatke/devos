use crate::driver::usb::speed::Speed;

pub struct Device {
    port: usize,
    speed: Speed,
}
