use core::future::poll_fn;
use core::task::{Context, Poll, Waker};
use crossbeam::queue::{ArrayQueue, SegQueue};

pub struct AsyncBoundedQueue<T> {
    pop_wakers: SegQueue<Waker>,
    push_wakers: SegQueue<Waker>,
    queue: ArrayQueue<T>,
}

impl<T> AsyncBoundedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            pop_wakers: SegQueue::new(),
            push_wakers: SegQueue::new(),
            queue: ArrayQueue::new(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub async fn pop(&self) -> T {
        poll_fn(|cx| self.poll_for_pop(cx)).await
    }

    pub fn pop_now(&self) -> Option<T> {
        self.queue.pop()
    }

    fn poll_for_pop(&self, cx: &mut Context) -> Poll<T> {
        if let Some(t) = self.pop_now() {
            self.wake_all_push_wakers();
            Poll::Ready(t)
        } else {
            self.pop_wakers.push(cx.waker().clone());
            Poll::Pending
        }
    }

    fn wake_all_push_wakers(&self) {
        while let Some(waker) = self.push_wakers.pop() {
            waker.wake();
        }
    }

    fn wake_all_pop_wakers(&self) {
        while let Some(waker) = self.pop_wakers.pop() {
            waker.wake();
        }
    }

    pub async fn push(&self, t: T) {
        poll_fn({
            let mut t_slot = Some(t);
            move |cx| {
                if let Some(t) = t_slot.take() {
                    match self.push_now(t) {
                        Ok(()) => {
                            self.wake_all_pop_wakers();
                            Poll::Ready(())
                        }
                        Err(t) => {
                            t_slot = Some(t);
                            self.push_wakers.push(cx.waker().clone());
                            Poll::Pending
                        }
                    }
                } else {
                    // We've already pushed the value, so this push is already done.
                    // We don't need to panic (although we could).
                    Poll::Ready(())
                }
            }
        })
        .await
    }

    pub fn push_now(&self, t: T) -> Result<(), T> {
        self.queue.push(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::future::executor::{block_on, Executor, TickResult};
    use alloc::sync::Arc;
    use core::sync::atomic::AtomicBool;
    use core::sync::atomic::Ordering::SeqCst;

    #[test]
    fn test_async_push() {
        let exec = Executor::new();
        let queue = Arc::new(AsyncBoundedQueue::<usize>::new(1));

        assert_eq!(0, queue.len());
        block_on(queue.push(5));

        assert_eq!(1, queue.len());
        exec.spawn({
            let queue = queue.clone();
            async move { queue.push(10).await }
        });

        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(1, queue.len());
        assert_eq!(TickResult::Idled, exec.execute_task());
        assert_eq!(1, queue.len());

        block_on(queue.pop());
        assert_eq!(0, queue.len());

        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(1, queue.len());
        assert_eq!(TickResult::Idled, exec.execute_task());
        assert_eq!(1, queue.len());

        assert_eq!(10, block_on(queue.pop()));

        exec.run_active_tasks_to_completion();
    }

    #[test]
    fn test_async_pop() {
        let exec = Executor::new();
        let queue = Arc::new(AsyncBoundedQueue::<usize>::new(1));
        let popped = Arc::new(AtomicBool::new(false));

        assert_eq!(0, queue.len());
        exec.spawn({
            let queue = queue.clone();
            let popped = popped.clone();
            async move {
                let res = queue.pop().await;
                popped.store(true, SeqCst);
                assert_eq!(5, res);
            }
        });

        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(0, queue.len());
        assert_eq!(TickResult::Idled, exec.execute_task());
        assert_eq!(0, queue.len());

        block_on(queue.push(5));
        assert_eq!(1, queue.len());
        assert_eq!(TickResult::Worked, exec.execute_task());
        assert_eq!(0, queue.len());
        assert_eq!(TickResult::Idled, exec.execute_task());
        assert_eq!(0, queue.len());

        assert!(popped.load(SeqCst));
    }
}
