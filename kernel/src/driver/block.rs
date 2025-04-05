use alloc::sync::Arc;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use spin::RwLock;

static BLOCK_DEVICES: OnceCell<RwLock<Vec<Arc<RwLock<dyn BlockDevice>>>>> = OnceCell::uninit();

pub fn block_devices() -> &'static RwLock<Vec<Arc<RwLock<dyn BlockDevice>>>> {
    BLOCK_DEVICES.get_or_init(Default::default)
}

pub fn register_block_device(device: impl BlockDevice + 'static) -> Arc<RwLock<dyn BlockDevice>> {
    let arc = Arc::new(RwLock::new(device));
    block_devices().write().push(arc.clone());
    arc
}

pub trait BlockDevice: Send + Sync {
    fn read(&mut self, block_id: usize, buf: &mut [u8]);
}
