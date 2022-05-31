extern crate alloc;

use core::{cell::UnsafeCell, fmt::Debug};
use alloc::{rc::Rc, sync::Arc};
use crate::{guards::*, movable::MovableMutex};

/// A mutually exclusive lock, attached to a value
pub struct Mutex<T: ?Sized> {
    pub(crate) inner: MovableMutex,
    pub(crate) data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex
    #[inline(always)]
    pub const fn new (data: T) -> Self {
        Self {
            inner: MovableMutex::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Creates a new mutex from it's parts
    #[inline(always)]
    pub const fn from_raw_parts (mutex: MovableMutex, data: T) -> Self {
        Self { 
            inner: mutex,
            data: UnsafeCell::new(data)
        }
    }

    /// Consumes the mutex and returns its underlying data
    #[inline(always)]
    pub fn into_inner (self) -> T {
        self.data.into_inner()
    }

    /// Consumes the mutex and returns its parts
    #[inline(always)]
    pub fn into_raw_parts (self) -> (MovableMutex, T) {
        (self.inner, self.data.into_inner())
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Attempts to lock the mutex, returning a [```MutexGuard```](crate::guards::MutexGuard) if it's successful, and ```None``` otherwise
    #[inline(always)]
    pub fn try_lock (&self) -> Option<MutexGuard<'_, T>> {
        if self.inner.try_lock() {
            return Some(MutexGuard {
                inner: self,
            })
        }

        None
    }

    /// Blocks the current thread until the mutex is acquired, returning a [```MutexGuard```](crate::guards::MutexGuard)
    #[cfg(feature = "sync")]
    #[inline(always)]
    pub fn lock_blocking (&self) -> MutexGuard<'_, T> {
        self.inner.lock_blocking();
        MutexGuard {
            inner: self,
        }
    }

    #[inline(always)]
    pub fn lock (&self) -> MutexFuture<'_, T> {
        MutexFuture {
            guard: Some(MutexGuard {
                inner: self,
            })
        }
    }

    #[inline(always)]
    pub fn try_lock_owned (self: Rc<Self>) -> Option<OwnedMutexGuard<T>> {
        if self.inner.try_lock() {
            return Some(OwnedMutexGuard {
                inner: self,
            })
        }

        None
    }

    #[cfg(feature = "sync")]
    #[inline(always)]
    pub fn lock_blocking_owned (self: Rc<Self>) -> OwnedMutexGuard<T> {
        self.inner.lock_blocking();
        OwnedMutexGuard {
            inner: self,
        }
    }

    #[inline(always)]
    pub fn lock_owned (self: Rc<Self>) -> OwnedMutexFuture<T> {
        OwnedMutexFuture {
            guard: Some(OwnedMutexGuard {
                inner: self,
            })
        }
    }

    #[inline(always)]
    pub fn try_lock_atomic (self: Arc<Self>) -> Option<AtomicMutexGuard<T>> {
        if self.inner.try_lock() {
            return Some(AtomicMutexGuard {
                inner: self,
            })
        }

        None
    }

    #[cfg(feature = "sync")]
    #[inline(always)]
    pub fn lock_blocking_atomic (self: Arc<Self>) -> AtomicMutexGuard<T> {
        self.inner.lock_blocking();
        AtomicMutexGuard {
            inner: self,
        }
    }

    #[inline(always)]
    pub fn lock_atomic (self: Arc<Self>) -> AtomicMutexFuture<T> {
        AtomicMutexFuture {
            guard: Some(AtomicMutexGuard {
                inner: self,
            })
        }
    }
}

impl<T> Debug for Mutex<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mutex").field("locked", &self.inner.locked).finish()
    }
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Sync> Sync for Mutex<T> {}