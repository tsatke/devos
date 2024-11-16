use crate::net::{DataLinkProtocol, Device, MacAddr};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use core::future::Future;
use core::net::{Ipv4Addr, Ipv6Addr};
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;
use derive_more::{Constructor, Display};
use foundation::falloc::vec::FVec;
use foundation::future::lock::FutureMutex;
use foundation::future::queue::AsyncBoundedQueue;

#[derive(Constructor)]
pub struct Frame(DataLinkProtocol, FVec<u8>);

impl Frame {
    pub fn protocol(&self) -> DataLinkProtocol {
        self.0
    }

    pub fn into_data(self) -> FVec<u8> {
        self.1
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Display)]
pub struct InterfaceId(usize);

impl InterfaceId {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT.fetch_add(1, Relaxed))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Addresses {
    ipv4_addr: Option<Ipv4Addr>,
    ipv6_addr: Option<Ipv6Addr>,
    mac_addr: MacAddr,
}

impl Addresses {
    pub fn ipv4_addr(&self) -> Option<Ipv4Addr> {
        self.ipv4_addr
    }

    pub fn ipv6_addr(&self) -> Option<Ipv6Addr> {
        self.ipv6_addr
    }

    pub fn mac_addr(&self) -> MacAddr {
        self.mac_addr
    }

    pub fn set_ipv4_addr(&mut self, ipv4_addr: Option<Ipv4Addr>) {
        self.ipv4_addr = ipv4_addr;
    }

    pub fn set_ipv6_addr(&mut self, ipv6_addr: Option<Ipv6Addr>) {
        self.ipv6_addr = ipv6_addr;
    }
}

pub struct Interface {
    id: InterfaceId,

    device: Arc<Box<dyn Device>>,
    addresses: Arc<FutureMutex<Addresses>>,
    protocol: DataLinkProtocol,

    tx_queue: Arc<AsyncBoundedQueue<Frame>>,
    rx_queue: Arc<AsyncBoundedQueue<Frame>>,
}

impl Debug for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interface")
            .field("id", &self.id)
            .field("protocol", &self.protocol)
            .finish_non_exhaustive()
    }
}

impl Interface {
    pub fn new(device: Box<dyn Device>) -> Interface {
        let mac_addr = device.mac_addr();
        let protocol = device.protocol();
        let device = Arc::new(device);
        Self {
            id: InterfaceId::new(),
            device,
            addresses: Arc::new(FutureMutex::new(Addresses {
                ipv4_addr: None,
                ipv6_addr: None,
                mac_addr,
            })),
            protocol,
            tx_queue: Arc::new(AsyncBoundedQueue::new(64)),
            rx_queue: Arc::new(AsyncBoundedQueue::new(64)),
        }
    }

    pub fn id(&self) -> InterfaceId {
        self.id
    }

    pub fn addresses(&self) -> &Arc<FutureMutex<Addresses>> {
        &self.addresses
    }

    pub fn protocol(&self) -> DataLinkProtocol {
        self.protocol
    }

    pub async fn send_frame(&self, frame: Frame) {
        self.tx_queue.push(frame).await
    }

    pub fn rx_queue(&self) -> &Arc<AsyncBoundedQueue<Frame>> {
        &self.rx_queue
    }

    pub fn work_rx_queue(&self) -> impl Future<Output = ()> + 'static {
        let device = self.device.clone();
        let rx_queue = self.rx_queue.clone();
        async move {
            loop {
                let frame = device.next_frame().await;
                rx_queue.push(frame).await;
            }
        }
    }

    pub fn work_tx_queue(&self) -> impl Future<Output = ()> + 'static {
        let device = self.device.clone();
        let tx_queue = self.tx_queue.clone();
        async move {
            loop {
                let frame = tx_queue.pop().await;
                device.write_frame(&frame.into_data()).await;
            }
        }
    }
}
