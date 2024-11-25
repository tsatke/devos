use crate::ethernet::{Ethernet, RawEthernetFrame};
use crate::interface::Interface;
use crate::Netstack;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::fmt::Debug;
use core::future::poll_fn;
use core::task::Poll;
use derive_more::Constructor;
use foundation::net::MacAddr;
use futures::future::BoxFuture;
use futures::FutureExt;
use log::{debug, error};

// make sure that [`Device`] is object safe
const _: Option<Box<dyn Device>> = None;

#[derive(Debug, Eq, PartialEq)]
pub enum RawDataLinkFrame {
    Ethernet(RawEthernetFrame),
}

pub trait Device: Send + Sync {
    fn read_frame(&self) -> BoxFuture<RawDataLinkFrame> {
        poll_fn(|cx| {
            if let Some(frame) = self.try_read_frame() {
                Poll::Ready(frame)
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .boxed()
    }
    fn write_frame(&self, frame: RawDataLinkFrame) -> BoxFuture<()> {
        let mut frame_slot = Some(frame);
        poll_fn(move |cx| {
            let Some(frame) = frame_slot.take() else {
                return Poll::Ready(());
            };

            if let Err(frame) = self.try_write_frame(frame) {
                frame_slot = Some(frame);
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .boxed()
    }

    fn mac_address(&self) -> MacAddr;
    fn try_read_frame(&self) -> Option<RawDataLinkFrame>;
    fn try_write_frame(&self, frame: RawDataLinkFrame) -> Result<(), RawDataLinkFrame>;
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
