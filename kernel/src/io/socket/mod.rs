use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::alloc::AllocError;
use core::sync::atomic::{AtomicUsize, Ordering};

use spin::Mutex;

pub use buffer::*;

mod buffer;

static SOCKETS: Mutex<BTreeMap<SocketId, Arc<SocketBuffer>>> = Mutex::new(BTreeMap::new());

pub fn create_socket() -> Result<SocketId, AllocError> {
    let id = SocketId::new();
    let buf = SocketBuffer::try_new()?;
    SOCKETS.lock().insert(id, Arc::new(buf));
    Ok(id)
}

pub fn get_socket(id: SocketId) -> Option<Arc<SocketBuffer>> {
    SOCKETS.lock().get(&id).cloned()
}

pub fn remove_socket(id: SocketId) {
    SOCKETS.lock().remove(&id);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SocketId(usize);

impl SocketId {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    pub fn into_usize(self) -> usize {
        self.0
    }
}
