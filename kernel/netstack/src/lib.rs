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
use log::debug;

pub mod arp;
pub mod device;
pub mod ethernet;
pub mod interface;
pub mod ip;
pub mod udp;

pub struct Netstack {
    executor: Executor<'static>,
    interfaces: FutureMutex<FVec<Arc<Interface>>>,

    arp_state: FutureMutex<arp::ArpCache>,
}

impl Netstack {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            executor: Executor::default(),
            interfaces: FutureMutex::default(),
            arp_state: FutureMutex::default(),
        })
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

    pub(crate) async fn handle_incoming_packet<'a, P, S>(
        self: &Arc<Self>,
        interface: Arc<Interface>,
        raw: S,
    ) -> Result<(), <P as Protocol>::Error>
    where
        P: Protocol,
        Arc<Netstack>: ProtocolSupport<P>,
        <P as Protocol>::Error: From<<P::Packet<'a> as TryFrom<S>>::Error> + 'static,
        P::Packet<'a>: TryFrom<S>,
        <P::Packet<'a> as TryFrom<S>>::Error: Error + 'static,
    {
        debug!("handling packet for protocol {}", P::name());
        let packet = P::Packet::try_from(raw)?;
        ProtocolSupport::<P>::protocol(self)
            .receive_packet(interface, packet)
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
    ($protocol:ty, $getter:ident) => {
        impl ProtocolSupport<$protocol> for Arc<Netstack> {
            fn protocol(&self) -> $protocol {
                <$protocol>::new(self.clone())
            }
        }

        impl Netstack {
            pub fn $getter(self: &Arc<Self>) -> $protocol
            where
                Arc<Self>: ProtocolSupport<$protocol>,
            {
                ProtocolSupport::<$protocol>::protocol(self)
            }
        }
    };
}

impl_protocol_support!(ethernet::Ethernet, ethernet);
impl_protocol_support!(arp::Arp, arp);
impl_protocol_support!(ip::Ip, ip);
impl_protocol_support!(udp::Udp, udp);

impl Tick for Netstack {
    fn tick(&self) -> TickResult {
        self.executor.tick()
    }
}

pub trait Packet {
    /// The size of the packet in bytes when serialized.
    fn wire_size(&self) -> usize;
}

pub trait Protocol {
    type Packet<'packet>: Packet;
    type Error: Error;

    fn name() -> &'static str;

    fn receive_packet<'a>(
        &self,
        interface: Arc<Interface>,
        packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::Error>>;

    fn send_packet(&self, packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::Error>>;
}
