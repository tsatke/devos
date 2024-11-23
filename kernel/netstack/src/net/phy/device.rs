use crate::net::{DataLinkProtocol, Frame};
use futures::future::BoxFuture;
use foundation::net::MacAddr;

pub trait Device: Send + Sync + 'static {
    fn mac_addr(&self) -> MacAddr;

    fn protocol(&self) -> DataLinkProtocol;

    fn next_frame(&self) -> BoxFuture<Frame>;

    fn write_frame(&self, buffer: &[u8]) -> BoxFuture<()>;
}

#[cfg(test)]
pub mod testing {
    use crate::net::{DataLinkProtocol, Device, Frame, MacAddr};
    use alloc::sync::Arc;
    use alloc::vec::Vec;
    use foundation::future::queue::AsyncBoundedQueue;
    use futures::future::BoxFuture;
    use futures::FutureExt;

    pub struct TestDevice {
        mac_addr: MacAddr,
        protocol: DataLinkProtocol,

        pub rx_queue: Arc<AsyncBoundedQueue<Vec<u8>>>,
        pub tx_queue: Arc<AsyncBoundedQueue<Vec<u8>>>,
    }

    impl TestDevice {
        pub fn create(mac_addr: MacAddr, protocol: DataLinkProtocol) -> Self {
            let rx_queue = Arc::new(AsyncBoundedQueue::new(16));
            let tx_queue = Arc::new(AsyncBoundedQueue::new(16));
            Self {
                mac_addr,
                protocol,
                rx_queue,
                tx_queue,
            }
        }
    }

    impl Device for TestDevice {
        fn mac_addr(&self) -> MacAddr {
            self.mac_addr
        }

        fn protocol(&self) -> DataLinkProtocol {
            self.protocol
        }

        fn next_frame(&self) -> BoxFuture<Frame> {
            async {
                let data = self.rx_queue.pop().await;
                Frame::new(self.protocol, data.into())
            }
            .boxed()
        }

        fn write_frame(&self, buffer: &[u8]) -> BoxFuture<()> {
            self.tx_queue.push(buffer.to_vec()).boxed()
        }
    }
}
