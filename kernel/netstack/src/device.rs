use crate::ethernet::{Ethernet, RawEthernetFrame};
use crate::interface::Interface;
use crate::Netstack;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::fmt::Debug;
use derive_more::Constructor;
use foundation::net::MacAddr;
use futures::future::BoxFuture;
use log::{debug, error};

// make sure that [`Device`] is object safe
const _: Option<Box<dyn Device>> = None;

#[derive(Debug, Eq, PartialEq)]
pub enum RawDataLinkFrame {
    Ethernet(RawEthernetFrame),
}

pub trait Device: Send + Sync {
    fn mac_address(&self) -> MacAddr;

    fn read_frame(&self) -> BoxFuture<RawDataLinkFrame>;
    fn write_frame(&self, frame: RawDataLinkFrame) -> BoxFuture<()>;
}

#[derive(Constructor)]
pub struct InterfaceWorker(Weak<Netstack>, Arc<Interface>);

impl InterfaceWorker {
    pub async fn run(&self) {
        loop {
            let frame = self.1.device().read_frame().await;
            let Some(net) = self.0.upgrade() else {
                debug!("netstack dropped, stopping interface worker");
                return;
            };
            if let Err(e) = match frame {
                RawDataLinkFrame::Ethernet(frame) => {
                    net.handle_incoming_packet::<Ethernet, _>(self.1.clone(), &frame)
                        .await
                }
            } {
                error!("error handling frame: {:?}", e);
            }
        }
    }
}
