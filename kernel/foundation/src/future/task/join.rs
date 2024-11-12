use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::channel::oneshot;

pub struct JoinHandle<T> {
    receiver: Pin<Box<oneshot::Receiver<T>>>,
}

impl<T> JoinHandle<T> {
    pub(crate) fn new(receiver: oneshot::Receiver<T>) -> Self {
        Self {
            receiver: Box::pin(receiver),
        }
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
    use crate::future::executor::{block_on, Executor};

    #[test]
    fn test_join_handle_no_panic_on_executor_drop() {
        let exec = Executor::new();
        let handle = exec.spawn(async { 1 }).unwrap();
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

        let exec = Executor::new();
        let result = exec.spawn(async {
            let a = increment(1).await;
            let b = increment(2).await;
            mul(a, b).await
        }).unwrap();

        let actual_result = exec.spawn(async move {
            let x = result.await;
            increment(x.unwrap_or(0)).await
        }).unwrap();

        exec.run_active_tasks_to_completion();

        let result = block_on(actual_result).unwrap();
        assert_eq!(result, 7);
    }
}