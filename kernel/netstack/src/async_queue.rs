use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use crossbeam::queue::{ArrayQueue, SegQueue};
use spin::Mutex;

pub struct AsyncArrayQueue<T> {
    queue: ArrayQueue<T>,
    pop_wakers: SegQueue<Waker>,
    push_wakers: SegQueue<Waker>,
}

impl<T> AsyncArrayQueue<T> {
    pub fn new(cap: usize) -> Self {
        Self {
            queue: ArrayQueue::new(cap),
            pop_wakers: SegQueue::new(),
            push_wakers: SegQueue::new(),
        }
    }

    pub fn pop(&self) -> impl Future<Output = T> + use<'_, T> {
        Pop {
            queue: &self.queue,
            pop_wakers: &self.pop_wakers,
            push_wakers: &self.push_wakers,
        }
    }

    pub fn push(&self, t: T) -> impl Future<Output = ()> + use<'_, T> {
        Push {
            elem: Mutex::new(Some(t)),
            queue: &self.queue,
            pop_wakers: &self.pop_wakers,
            push_wakers: &self.push_wakers,
        }
    }
}

pub struct Pop<'a, T> {
    queue: &'a ArrayQueue<T>,
    pop_wakers: &'a SegQueue<Waker>,
    push_wakers: &'a SegQueue<Waker>,
}

impl<'a, T> Future for Pop<'a, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(t) = self.queue.pop() {
            // we popped an element, so there may be space for pushing, thus
            // we wake all the pushers
            while let Some(waker) = self.push_wakers.pop() {
                waker.wake();
            }

            return Poll::Ready(t);
        }

        // no element, so register a waker and go to sleep
        self.pop_wakers.push(cx.waker().clone());
        Poll::Pending
    }
}

pub struct Push<'a, T> {
    elem: Mutex<Option<T>>,
    queue: &'a ArrayQueue<T>,
    pop_wakers: &'a SegQueue<Waker>,
    push_wakers: &'a SegQueue<Waker>,
}

impl<'a, T> Future for Push<'a, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let elem = self.elem.lock().take().unwrap();
        match self.queue.push(elem) {
            Ok(_) => Poll::Ready(()),
            Err(e) => {
                let _ = self.elem.lock().insert(e);
                self.push_wakers.push(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
