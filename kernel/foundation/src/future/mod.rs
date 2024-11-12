use core::future::poll_fn;
use core::task::Poll;

pub mod lock;

pub mod executor;

pub async fn yield_now() {
    let mut yielded = false;
    poll_fn(move |cx| {
        if yielded {
            return Poll::Ready(());
        }
        yielded = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }).await
}

#[cfg(test)]
pub mod testing {
    use core::future::Future;
    use core::pin::Pin;
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering::SeqCst;
    use core::task::{Context, Poll};

    pub struct Times<T, const N: usize> {
        current: AtomicUsize,
        res: T,
    }

    impl<T, const N: usize> Times<T, N> {
        pub fn new(res: T) -> Self {
            Self {
                current: AtomicUsize::new(0),
                res,
            }
        }
    }

    impl<T, const N: usize> Future for Times<T, N>
    where
        T: Copy,
    {
        type Output = T;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            loop {
                let current = self.current.load(SeqCst);
                if current < N {
                    if self.current.compare_exchange(current, current + 1, SeqCst, SeqCst).is_err() {
                        continue;
                    }
                    self.current.fetch_add(1, SeqCst);
                    cx.waker().wake_by_ref();
                    break Poll::Pending;
                } else {
                    break Poll::Ready(self.res);
                }
            }
        }
    }
}
