use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::hint::spin_loop;
use core::pin::Pin;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::SeqCst;
use core::task::{Context, Poll, Waker};

pub trait RelaxStrategy {
    fn relax();
}

pub struct Spin;

impl RelaxStrategy for Spin {
    fn relax() {
        spin_loop();
    }
}

pub fn block_on<T, R: RelaxStrategy>(fut: impl Future<Output = T> + 'static) -> T {
    SingleTaskExecutor::new(fut).execute::<R>()
}

pub struct SingleTaskExecutor<T> {
    waker: Waker,
    task: Pin<Box<dyn Future<Output = T>>>,
    sleeping: Arc<AtomicBool>,
}

impl<T> SingleTaskExecutor<T> {
    pub fn new(fut: impl Future<Output = T> + 'static) -> Self {
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
    use core::sync::atomic::AtomicUsize;

    struct Times<T, const N: usize> {
        current: AtomicUsize,
        res: T,
    }

    impl<T, const N: usize> Future for Times<T, N>
    where
        T: Copy,
    {
        type Output = T;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.current.load(SeqCst) < N {
                self.current.fetch_add(1, SeqCst); // TOCTOU - not relevant in these tests
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(self.res)
            }
        }
    }

    #[test]
    fn test_block_on() {
        let t = Times::<&str, 142> {
            current: AtomicUsize::new(0),
            res: "hello",
        };
        assert_eq!(block_on::<_, Spin>(async { t.await }), "hello");
    }
}
