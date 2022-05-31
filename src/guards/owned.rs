extern crate alloc;
use core::{ops::{Deref, DerefMut}, task::Poll};
use alloc::rc::Rc;
use futures::{future::FusedFuture, Future};
use crate::{Mutex};

#[repr(transparent)]
pub struct OwnedMutexGuard<T: ?Sized> {
    pub(crate) inner: Rc<Mutex<T>>
}

impl<T: ?Sized> OwnedMutexGuard<T> {
    #[inline(always)]
    pub fn unlock (self) {}
}

impl<T: ?Sized> Deref for OwnedMutexGuard<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.data.get() }
    }
}

impl<T: ?Sized> DerefMut for OwnedMutexGuard<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.data.get() }
    }
}

impl<T: ?Sized> Drop for OwnedMutexGuard<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.inner.inner.unlock(); }
    }
}

/// Future that resolves to an owned mutex guard
#[repr(transparent)]
pub struct OwnedMutexFuture<T: ?Sized> {
    pub(crate) guard: Option<OwnedMutexGuard<T>>
}

impl<T: ?Sized> Future for OwnedMutexFuture<T> {
    type Output = OwnedMutexGuard<T>;

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

impl<'a, T: ?Sized> FusedFuture for OwnedMutexFuture<T> {
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.guard.is_none()
    }
}