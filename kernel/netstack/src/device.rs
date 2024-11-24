use crate::ethernet::{Ethernet, RawEthernetFrame};
use crate::Netstack;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use derive_more::Constructor;
use foundation::net::MacAddr;
use futures::future::BoxFuture;
use log::error;

// make sure that [`Device`] is object safe
const _: Option<Box<dyn Device>> = None;

#[derive(Debug, Eq, PartialEq)]
pub enum RawDataLinkFrame {
    Ethernet(RawEthernetFrame),
}

pub trait Device: Debug + Send + Sync {
    fn mac_address(&self) -> MacAddr;
    fn read_frame(&self) -> BoxFuture<RawDataLinkFrame>;
    fn write_frame(&self, frame: RawDataLinkFrame) -> BoxFuture<()>;
}

#[derive(Constructor)]
pub struct DeviceWorker(Arc<Netstack>, Arc<Box<dyn Device>>);

impl DeviceWorker {
    pub async fn run(&self) {
        let net = &self.0;
        loop {
            let frame = self.1.read_frame().await;
            if let Err(e) = match frame {
                RawDataLinkFrame::Ethernet(frame) => net.handle_packet::<Ethernet, _>(frame).await,
            } {
                error!("error handling frame: {:?}", e);
            }
        }
    }
}
