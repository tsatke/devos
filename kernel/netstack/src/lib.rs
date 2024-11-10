#![no_std]
#![feature(allocator_api)]
extern crate alloc;

use crate::executor::{ExecuteResult, Executor};
use crate::net::{Buffer, Interface, IpCidr};
use alloc::boxed::Box;
use alloc::sync::Arc;
use crossbeam::queue::ArrayQueue;
use derive_more::From;
use device::Device;
use foundation::falloc::vec::FVec;
use futures::StreamExt;
use spin::Mutex;

mod device;
pub mod executor;
mod net;

pub struct NetStack {
    executor: Executor,
    routing: Mutex<FVec<Route>>,
    rx_queue: Arc<ArrayQueue<Buffer>>,
}

impl NetStack {
    pub fn register_device(&self, cidr: IpCidr, device: Box<dyn Device>) -> Result<(), ()> {
        let interface = Interface::new(device);
        let frame_stream = interface.frames().map_err(|_| ())?;

        let route = Route(cidr, interface);
        self.routing.lock().try_push(route).map_err(|_| ())?;

        self.executor.spawn({
            let rx_queue = self.rx_queue.clone();
            let mut frame_stream = frame_stream.fuse();
            async move {
                while let Some(buf) = frame_stream.next().await {
                    match rx_queue.push(buf) {
                        Err(_) => {
                            // FIXME: log that we dropped a frame and move on
                            panic!("dropping a packet - we should just log this and continue")
                        }
                        _ => {}
                    }
                }
                // no more frames will come from this device, so we can exit the task
            }
        })?;

        Ok(())
    }

    pub fn execute_step(&self) -> ExecuteResult {
        self.executor.execute_task()
    }
}

#[derive(From)]
pub struct Route(IpCidr, Interface);
