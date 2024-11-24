#![no_std]
extern crate alloc;

use crate::device::Device;
use crate::interface::Interface;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::error::Error;
use device::InterfaceWorker;
use foundation::falloc::vec::FVec;
use foundation::future::executor::{Executor, Tick, TickResult};
use foundation::future::lock::FutureMutex;
use futures::future::BoxFuture;

pub mod arp;
pub mod device;
pub mod ethernet;
pub mod interface;
pub mod ip;
pub mod udp;

pub struct Netstack {
    executor: Executor<'static>,
    interfaces: FutureMutex<FVec<Arc<Interface>>>,
}

impl Netstack {
    pub fn new() -> Arc<Self> {
        let s = Arc::new(Self {
            executor: Executor::new(),
            interfaces: FutureMutex::new(FVec::new()),
        });

        s
    }

    pub async fn add_device(self: &Arc<Self>, device: Box<dyn Device>) {
        let interface = Arc::new(Interface::new(device));
        self.interfaces
            .lock()
            .await
            .try_push(interface.clone())
            .unwrap(); // TODO: handle error

        self.executor.spawn({
            let netstack = self.clone();
            let interface = interface.clone();
            async move {
                InterfaceWorker::new(netstack, interface).run().await;
            }
        });
    }

    pub(crate) async fn handle_packet<'a, P, S>(
        self: &Arc<Self>,
        raw: S,
    ) -> Result<(), <P as Protocol>::Error>
    where
        P: Protocol,
        Arc<Netstack>: ProtocolSupport<P>,
        <P as Protocol>::Error: From<<P::Packet<'a> as TryFrom<S>>::Error> + 'static,
        P::Packet<'a>: TryFrom<S>,
        <P::Packet<'a> as TryFrom<S>>::Error: Error + 'static,
    {
        let packet = P::Packet::try_from(raw)?;
        ProtocolSupport::<P>::protocol(self)
            .process_packet(packet)
            .await?;
        Ok(())
    }
}

pub trait ProtocolSupport<P>
where
    P: Protocol,
{
    fn protocol(&self) -> P;
}

macro_rules! impl_protocol_support {
    ($protocol:ty) => {
        impl ProtocolSupport<$protocol> for Arc<Netstack> {
            fn protocol(&self) -> $protocol {
                <$protocol>::new(self.clone())
            }
        }
    };
}

impl_protocol_support!(ethernet::Ethernet);
impl_protocol_support!(arp::Arp);
impl_protocol_support!(ip::Ip);
impl_protocol_support!(udp::Udp);

impl Tick for Netstack {
    fn tick(&self) -> TickResult {
        self.executor.tick()
    }
}

pub trait Protocol {
    type Packet<'packet>;
    type Error: Error;

    fn process_packet<'a>(
        &self,
        packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::Error>>;

    fn send_packet(&self, packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::Error>>;
}
