use alloc::boxed::Box;
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use core::error::Error;
use foundation::future::executor::block_on;
use netstack::device::Device;
use netstack::Netstack;

static NETSTACK: OnceCell<Arc<Netstack>> = OnceCell::uninit();

pub fn register_nic(nic: Box<dyn Device>) -> Result<(), Box<dyn Error>> {
    block_on(netstack().add_device(nic))?;
    Ok(())
}

pub fn netstack() -> &'static Arc<Netstack> {
    NETSTACK.get_or_init(Netstack::new)
}
