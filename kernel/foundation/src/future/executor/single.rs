use crate::future::lock::{RelaxStrategy, Spin};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::SeqCst;
use core::task::{Context, Poll, Waker};

pub fn block_on<T>(fut: impl Future<Output=T>) -> T {
    SingleTaskExecutor::new(fut).execute::<Spin>()
}

pub struct SingleTaskExecutor<T> {
    waker: Waker,
    task: Pin<Box<T>>,
    sleeping: Arc<AtomicBool>,
}

impl<F, T> SingleTaskExecutor<F>
where
    F: Future<Output=T>,
{
    pub fn new(fut: F) -> Self {
        let sleeping = Arc::new(AtomicBool::new(false));
        let waker = Waker::from(Arc::new(SingleTaskWaker {
            executor_sleeping: sleeping.clone(),
        }));
        Self {
            waker,
            task: Box::pin(fut),
            sleeping: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn execute<R: RelaxStrategy>(&mut self) -> T {
        loop {
            while self.sleeping.load(SeqCst) {
                R::relax();
            }

            let mut context = Context::from_waker(&self.waker);
            match self.task.as_mut().poll(&mut context) {
                Poll::Ready(t) => break t,
                Poll::Pending => {}
            }
        }
    }
}

struct SingleTaskWaker {
    executor_sleeping: Arc<AtomicBool>,
}

impl Wake for SingleTaskWaker {
    fn wake(self: Arc<Self>) {
        self.executor_sleeping.store(false, SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::future::testing::Times;

    #[test]
    fn test_block_on() {
        let t = Times::<_, 12>::new("hello");
        assert_eq!(block_on(async { t.await }), "hello");
    }
}
