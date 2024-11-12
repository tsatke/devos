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
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.as_mut().poll(cx) {
            Poll::Ready(v) => Poll::Ready(v.unwrap()), // we must never drop the sender
            Poll::Pending => Poll::Pending,
        }
    }
}
