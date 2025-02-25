use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::future::Future;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst};
use core::sync::atomic::{compiler_fence, AtomicBool};
use core::task::{Context, Poll, Waker};
use crossbeam::queue::SegQueue;

pub trait RelaxStrategy {
    fn relax();
}

#[derive(Default)]
pub struct Spin;

impl RelaxStrategy for Spin {
    fn relax() {
        spin_loop();
    }
}

pub struct FutureMutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
    wakers: SegQueue<Waker>,
}

unsafe impl<T: Send> Sync for FutureMutex<T> {}
unsafe impl<T: Send> Send for FutureMutex<T> {}

impl<T: Default> Default for FutureMutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for FutureMutex<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<T> FutureMutex<T> {
    pub const fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(t),
            wakers: SegQueue::new(),
        }
    }

    pub fn lock(&self) -> FutexMutexGuardFuture<'_, T> {
        FutexMutexGuardFuture::new(self)
    }

    pub fn try_lock(&self) -> Option<FutureMutexGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Acquire, Relaxed)
            .is_ok()
        {
            Some(FutureMutexGuard {
                mutex: self,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }

    pub fn lock_sync<R: RelaxStrategy>(&self) -> FutureMutexGuard<'_, T> {
        loop {
            if let Some(guard) = self.try_lock() {
                break guard;
            }
            R::relax();
        }
    }
}

pub struct FutureMutexGuard<'a, T> {
    mutex: &'a FutureMutex<T>,
    data: *mut T,
}

impl<T> Debug for FutureMutexGuard<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as Debug>::fmt(self, f)
    }
}

unsafe impl<T: Sync> Sync for FutureMutexGuard<'_, T> {}
unsafe impl<T: Send> Send for FutureMutexGuard<'_, T> {}

impl<T> Deref for FutureMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for FutureMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}
impl<T> Drop for FutureMutexGuard<'_, T> {
    fn drop(&mut self) {
        // Order matters here, this needs to happen before waking up other tasks.
        // Consider this.
        // We wake up another tasks first. Then, we are preempted.
        // The other task sees the lock as still locked and goes
        // back to sleep.
        // We now unlock the lock. All waiting tasks will keep waiting.
        // The next task that locks the lock will solve this, but there's
        // no guarantee that there is such a task.
        //
        // Something that would relax the situation, but not solve it, would
        // be the executor sporadically waking tasks up that were not woken,
        // but we mustn't rely on that.
        self.mutex.locked.store(false, Release);

        compiler_fence(SeqCst); // TODO: is this correct/necessary?

        // only one task will get the lock, so we only need to wake one
        if let Some(waker) = self.mutex.wakers.pop() {
            waker.wake_by_ref()
        }
    }
}

pub struct FutexMutexGuardFuture<'a, T> {
    mutex: &'a FutureMutex<T>,
}

impl<'a, T> FutexMutexGuardFuture<'a, T> {
    fn new(mutex: &'a FutureMutex<T>) -> Self {
        Self { mutex }
    }
}

impl<'a, T> Future for FutexMutexGuardFuture<'a, T> {
    type Output = FutureMutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We MUST try to acquire the lock if we are polled, and we can't use
        // a weak try_lock here. A dropped mutex guard will only wake one
        // task, and if that task does not acquire a mutex, no other task will
        // be woken.
        //
        // If, in the future, for some reason, we don't want to acquire the task,
        // although we think we could, we have to wake another task from
        // the mutex ourselves.
        if let Some(guard) = self.mutex.try_lock() {
            return Poll::Ready(guard);
        }

        // here, the lock might already be available again, but checking again
        // after registering the waker would require some expensive bookkeeping
        // in the guard and the mutex, so we just take the loss and re-queue

        self.mutex.wakers.push(cx.waker().clone());

        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::future::executor::{Executor, Tick, TickResult};
    use alloc::sync::Arc;

    #[test]
    fn test_mutex_lock_unlock() {
        let executor = Executor::default();
        let mutex = Arc::new(FutureMutex::new(0_usize));
        let guard = mutex.try_lock().unwrap();

        const TASKS: usize = 10;

        for _ in 0..TASKS {
            executor.spawn({
                let mutex = mutex.clone();
                async move {
                    let mut guard = mutex.lock().await;
                    *guard += 1;
                }
            });
        }

        assert_eq!(*guard, 0);
        // every task should attempt to get the lock, then put itself to sleep
        for _ in 0..TASKS {
            let res = executor.tick();
            assert_eq!(res, TickResult::Worked)
        }
        // all tasks should be asleep, the executor should only idle now
        for _ in 0..(TASKS * 10) {
            let res = executor.tick();
            assert_eq!(res, TickResult::Idled)
        }
        assert_eq!(*guard, 0);

        drop(guard);
        for _ in 0..TASKS {
            let res = executor.tick();
            assert_eq!(res, TickResult::Worked);
        }
        assert_eq!(executor.tick(), TickResult::Idled);

        let guard = mutex.try_lock().unwrap();
        assert_eq!(*guard, TASKS);
    }
}
