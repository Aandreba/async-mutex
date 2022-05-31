extern crate alloc;
use core::{sync::atomic::Ordering, task::{Poll}, future::Future, fmt::Debug};
use alloc::{rc::Rc, sync::Arc};
use crate::{Flag, FALSE, TRUE, queue::Queue, waker::Waker};

/// A mutex that is not attached to any value
pub struct MovableMutex {
    pub(crate) locked: Flag,
    pub(crate) queue: Queue
}

impl MovableMutex {
    /// Creates a new mutex
    #[inline(always)]
    pub const fn new () -> Self {
        Self { 
            locked: Flag::new(FALSE),
            queue: Queue::new()
        }
    }

    /// Creates a new mutex that starts locked
    #[inline(always)]
    pub const fn locked () -> Self {
        Self { 
            locked: Flag::new(TRUE),
            queue: Queue::new()
        }
    }

    /// Attempts to lock the mutex, returning ```true``` if it's successful, and ```false``` otherwise
    #[inline(always)]
    pub fn try_lock (&self) -> bool {
        self.locked.compare_exchange(FALSE, TRUE, Ordering::Acquire, Ordering::Acquire).is_ok()
    }

    /// Blocks the current thread until the mutex is acquired
    #[cfg(feature = "sync")]
    #[inline(always)]
    pub fn lock_blocking (&self) {
        loop {
            if self.try_lock() { return; }
            let waker = Arc::new(Flag::new(FALSE));
            self.queue.push(Waker::Sync(waker.clone()));

            while waker.load(Ordering::Acquire) == FALSE { }
            core::hint::spin_loop();
        }
    }

    /// Returns a future that resolves when the mutex is acquired by reference
    #[inline(always)]
    pub fn lock (&self) -> MovableMutexFuture<'_> {
        MovableMutexFuture {
            mutex: self
        }
    }

    /// Returns a future that resolves when the mutex is acquired by [```Rc```](alloc::rc::Rc)
    #[inline(always)]
    pub fn lock_owned (self: Rc<Self>) -> OwnedMovableMutexFuture {
        OwnedMovableMutexFuture {
            mutex: self
        }
    }

    /// Returns a future that resolves when the mutex is acquired by [```Arc```](alloc::sync::Arc)
    #[inline(always)]
    pub fn lock_atomic (self: Arc<Self>) -> AtomicMovableMutexFuture {
        AtomicMovableMutexFuture {
            mutex: self
        }
    }

    /// Unlocks the mutex, without checking if this thread was it's owner
    #[inline(always)]
    pub unsafe fn unlock (&self) {
        #[cfg(debug_assertions)]
        assert_eq!(self.locked.swap(FALSE, Ordering::Release), TRUE);
        #[cfg(not(debug_assertions))]
        self.locked.store(FALSE, Ordering::Release);
        self.queue.wake();
    }
}

impl Debug for MovableMutex {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MovableMutex").field("locked", &self.locked).finish()
    }
}

/// Future of [```lock```](MovableMutex::lock) 
#[repr(transparent)]
pub struct MovableMutexFuture<'a> {
    mutex: &'a MovableMutex
}

/// Future of [```lock_owned```](MovableMutex::lock_owned) 
#[repr(transparent)]
pub struct OwnedMovableMutexFuture {
    mutex: Rc<MovableMutex>
}

/// Future of  [```lock_atomic```](MovableMutex::lock_atomic) 
#[repr(transparent)]
pub struct AtomicMovableMutexFuture {
    mutex: Arc<MovableMutex>
}

impl<'a> Future for MovableMutexFuture<'a> {
    type Output = ();

    #[inline(always)]
    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if self.mutex.try_lock() {
            return Poll::Ready(());
        }

        self.mutex.queue.push(cx.waker().clone().into());
        Poll::Pending
    }
}

impl Future for OwnedMovableMutexFuture {
    type Output = ();

    #[inline(always)]
    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if self.mutex.try_lock() {
            return Poll::Ready(());
        }

        self.mutex.queue.push(cx.waker().clone().into());
        Poll::Pending
    }
}

impl Future for AtomicMovableMutexFuture {
    type Output = ();

    #[inline(always)]
    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if self.mutex.try_lock() {
            return Poll::Ready(());
        }

        self.mutex.queue.push(cx.waker().clone().into());
        Poll::Pending
    }
}