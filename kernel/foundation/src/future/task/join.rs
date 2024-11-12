use alloc::sync::Arc;
use spin::Mutex;

pub struct JoinHandle<T> {
    // TODO: use something like oneshot::channel::<T>() instead of a Mutex<Option<T>>
    result: Arc<Mutex<Option<T>>>,
}

impl<T> Default for JoinHandle<T> {
    fn default() -> Self {
        Self {
            result: Arc::new(Mutex::new(None)),
        }
    }
}

impl<T> JoinHandle<T> {
    pub(crate) fn result(&self) -> Arc<Mutex<Option<T>>> {
        self.result.clone()
    }

    pub fn get_result(&self) -> Option<T> {
        self.result.lock().take()
    }
}
