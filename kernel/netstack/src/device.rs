use crate::ethernet::{Ethernet, RawEthernetFrame};
use crate::Netstack;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use derive_more::Constructor;
use futures::future::BoxFuture;
use log::error;

// make sure that [`Device`] is object safe
const _: Option<Box<dyn Device>> = None;

#[derive(Debug, Eq, PartialEq)]
pub enum RawDataLinkFrame {
    Ethernet(RawEthernetFrame),
}

pub trait Device: Debug + Send + Sync {
    fn read_frame(&self) -> BoxFuture<RawDataLinkFrame>;
    fn write_frame(&self, frame: RawDataLinkFrame) -> BoxFuture<()>;
}

#[derive(Constructor)]
pub struct DeviceWorker(Arc<Netstack>, Arc<Box<dyn Device>>);

impl DeviceWorker {
    pub async fn run(&self) {
        loop {
            let frame = self.1.read_frame().await;
            let res = match frame {
                RawDataLinkFrame::Ethernet(frame) => {
                    self.0.handle_packet::<Ethernet, _>(frame).await
                }
            };
            if let Err(e) = res {
                error!("error handling frame: {:?}", e);
            }
        }
    }
}
