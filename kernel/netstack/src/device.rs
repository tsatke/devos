use crate::ethernet::{Ethernet, RawEthernetFrame};
use crate::interface::Interface;
use crate::Netstack;
use alloc::sync::{Arc, Weak};
use core::fmt::Debug;
use derive_more::Constructor;
use log::{debug, error};

#[derive(Debug, Eq, PartialEq)]
pub enum RawDataLinkFrame {
    Ethernet(RawEthernetFrame),
}

#[derive(Constructor)]
pub struct InterfaceWorker(Weak<Netstack>, Arc<Interface>);

impl InterfaceWorker {
    pub async fn run(&self) {
        loop {
            let Some(net) = self.0.upgrade() else {
                debug!("netstack dropped, stopping interface worker");
                return;
            };

            let frame = self.1.rx_queue().pop().await;
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
