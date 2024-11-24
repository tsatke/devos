use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Acquire, Release};
use core::task::{Context, Poll};
use futures::channel::oneshot;
use futures::future::FusedFuture;

pub struct JoinHandle<T> {
    receiver: Pin<Box<oneshot::Receiver<T>>>,
    should_cancel: Arc<AtomicBool>,
}

impl<T> JoinHandle<T> {
    pub(crate) fn new(receiver: oneshot::Receiver<T>, should_cancel: Arc<AtomicBool>) -> Self {
        Self {
            receiver: Box::pin(receiver),
            should_cancel,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.should_cancel.load(Acquire) || self.receiver.is_terminated()
    }

    pub fn cancel(self) {
        self.should_cancel.store(true, Release);
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Option<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.as_mut().poll(cx) {
            Poll::Ready(v) => Poll::Ready(v.ok()),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::future::executor::{block_on, Executor, TickResult};
    use crate::future::testing::Times;
    use crate::future::yield_now;
    use alloc::sync::Arc;
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering::{Acquire, SeqCst};

    #[test]
    fn test_join_handle_no_panic_on_executor_drop() {
        let exec = Executor::default();
        let handle = exec.spawn(async { 1 });
        drop(exec);
        assert_eq!(None, block_on(handle));
    }

    #[test]
    fn test_join_handle_await() {
        async fn increment(a: usize) -> usize {
            a + 1
        }
        async fn mul(a: usize, b: usize) -> usize {
            a * b
        }

        let exec = Executor::default();
        let result = exec.spawn(async {
            let a = increment(1).await;
            let b = increment(2).await;
            mul(a, b).await
        });

        let actual_result = exec.spawn(async move {
            let x = result.await;
            increment(x.unwrap_or(0)).await
        });

        exec.run_active_tasks_to_completion();

        let result = block_on(actual_result).unwrap();
        assert_eq!(result, 7);
    }

    #[test]
    fn test_join_handle_cancel() {
        let exec = Executor::default();
        let handle = exec.spawn(Times::<_, 100>::new(()));
        assert!(!handle.is_finished());

        for _ in 0..5 {
            assert_eq!(TickResult::Worked, exec.execute_task());
        }
        assert!(!handle.is_finished());
        handle.cancel();

        assert_eq!(TickResult::Idled, exec.execute_task());
    }

    #[test]
    fn test_drop_join_handle_doesnt_affect_task_execution() {
        let counter = Arc::new(AtomicUsize::new(0));

        let exec = Executor::default();
        let handle = exec.spawn({
            let counter = counter.clone();
            async move {
                for _ in 0..5 {
                    counter.fetch_add(1, SeqCst);
                    yield_now().await;
                }
            }
        });

        assert_eq!(counter.load(Acquire), 0);
        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(counter.load(Acquire), 1);
        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(counter.load(Acquire), 2);

        drop(handle);

        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(counter.load(Acquire), 3);
        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(counter.load(Acquire), 4);
        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(counter.load(Acquire), 5);
        assert_eq!(TickResult::Worked, exec.execute_task()); // after yielding

        assert_eq!(TickResult::Idled, exec.execute_task());
    }
}
