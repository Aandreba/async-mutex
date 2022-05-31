use core::{ops::{Deref, DerefMut}, task::Poll};
use futures::{Future, future::FusedFuture};
use crate::{Mutex};

#[repr(transparent)]
pub struct MutexGuard<'a, T: ?Sized> {
    pub(crate) inner: &'a Mutex<T>
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    #[inline(always)]
    pub fn unlock (self) {}
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.inner.inner.unlock(); }
    }
}

/// Future that resolves to an owned mutex guard
#[repr(transparent)]
pub struct MutexFuture<'a, T: ?Sized> {
    pub(crate) guard: Option<MutexGuard<'a, T>>
}

impl<'a, T: ?Sized> Future for MutexFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        let guard = if let Some(ref mut guard) = self.guard { guard } else { panic!("Mutex future already consumed") };
        if guard.inner.inner.try_lock() {
            let guard = core::mem::take(&mut self.guard).unwrap();
            return Poll::Ready(guard);
        }

        guard.inner.inner.queue.push(cx.waker().clone().into());
        Poll::Pending
    }
}

impl<'a, T: ?Sized> FusedFuture for MutexFuture<'a, T> {
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.guard.is_none()
    }
}

flat_mod!(owned, atomic);