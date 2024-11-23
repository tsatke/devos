#![no_std]
extern crate alloc;

use crate::device::Device;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::error::Error;
use device::DeviceWorker;
use foundation::falloc::vec::FVec;
use foundation::future::executor::{Executor, Tick, TickResult};
use foundation::future::lock::FutureMutex;
use futures::future::BoxFuture;

pub mod arp;
pub mod device;
pub mod ethernet;
pub mod ip;

pub struct Netstack {
    executor: Executor<'static>,
    devices: FutureMutex<FVec<Arc<Box<dyn Device>>>>,
}

impl Netstack {
    pub fn new() -> Arc<Self> {
        let s = Arc::new(Self {
            executor: Executor::new(),
            devices: FutureMutex::new(FVec::new()),
        });

        s
    }

    pub async fn add_device(self: &Arc<Self>, device: Box<dyn Device>) {
        let device = Arc::new(device);
        self.devices.lock().await.try_push(device.clone()).unwrap(); // TODO: handle error
        self.executor.spawn({
            let netstack = self.clone();
            let device = device.clone();
            async move {
                DeviceWorker::new(netstack, device).run().await;
            }
        });
    }

    pub(crate) async fn handle_packet<'a, P, S>(
        self: &Arc<Self>,
        raw: S,
    ) -> Result<(), Box<dyn Error + 'static>>
    where
        P: Protocol,
        <P as Protocol>::ProcessError: 'static,
        Arc<Netstack>: ProtocolSupport<P>,
        P::Packet<'a>: TryFrom<S> + 'static,
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

impl_protocol_support!(arp::Arp);
impl_protocol_support!(ip::Ip);

impl Tick for Netstack {
    fn tick(&self) -> TickResult {
        self.executor.tick()
    }
}

pub trait Protocol {
    type Packet<'packet>;
    type ProcessError: Error;
    type SendError: Error;

    fn process_packet(&self, packet: Self::Packet<'_>)
        -> BoxFuture<Result<(), Self::ProcessError>>;

    fn send_packet(&self, packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::SendError>>;
}
