use core::{sync::atomic::{AtomicU8, Ordering}, mem::MaybeUninit, cell::UnsafeCell};

pub struct AtomicCell<T> {
    state: State,
    v: UnsafeCell<MaybeUninit<T>>
}

impl<T> AtomicCell<T> {
    #[inline(always)]
    pub const fn new (v: T) -> Self {
        Self {
            state: State::new(SOME),
            v: UnsafeCell::new(MaybeUninit::new(v))
        }
    }

    #[inline(always)]
    pub const fn empty () -> Self {
        Self {
            state: State::new(NONE),
            v: UnsafeCell::new(MaybeUninit::uninit())
        }
    }

    #[inline(always)]
    pub fn try_get_ref (&self) -> Option<&T> {
        match self.state.compare_exchange(SOME, WORKING, Ordering::Acquire, Ordering::Acquire) {
            Ok(_) => unsafe {
                let value = (&*self.v.get()).assume_init_ref();
                #[cfg(debug_assertions)]
                assert_eq!(self.state.swap(SOME, Ordering::Release), WORKING);
                #[cfg(not(debug_assertions))]
                self.state.store(SOME, Ordering::Release);
                Some(value)
            },

            Err(_) => None
        }
    }

    #[inline(always)]
    pub fn try_get (&self) -> Option<T> {
        match self.state.compare_exchange(SOME, WORKING, Ordering::Acquire, Ordering::Acquire) {
            Ok(_) => unsafe {
                let value = core::mem::replace(&mut *self.v.get(), MaybeUninit::uninit()).assume_init();
                #[cfg(debug_assertions)]
                assert_eq!(self.state.swap(NONE, Ordering::Release), WORKING);
                #[cfg(not(debug_assertions))]
                self.state.store(NONE, Ordering::Release);
                Some(value)
            },

            Err(_) => None
        }
    }

    #[inline(always)]
    pub fn try_set (&self, v: T) -> Option<T> {
        match self.state.compare_exchange(NONE, WORKING, Ordering::Acquire, Ordering::Acquire) {
            Ok(_) => unsafe {
                (&mut *self.v.get()).write(v);
                #[cfg(debug_assertions)]
                assert_eq!(self.state.swap(SOME, Ordering::Release), WORKING);
                #[cfg(not(debug_assertions))]
                self.state.store(SOME, Ordering::Release);
                None
            },

            Err(_) => Some(v)
        }
    }
}

impl<T> Drop for AtomicCell<T> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.state.load(Ordering::Relaxed) == SOME {
            unsafe { self.v.get_mut().assume_init_drop() }
        }
    }
}

unsafe impl<T: Send> Send for AtomicCell<T> {}
unsafe impl<T: Sync> Sync for AtomicCell<T> {}

cfg_if::cfg_if! {
    if #[cfg(target_has_atomic = "8")] {
        type State = AtomicU8;
        const NONE : u8 = 0;
        const WORKING : u8 = 1;
        const SOME : u8 = 2;
    } else if #[cfg(target_has_atomic = "16")] {
        type State = AtomicU16;
        const NONE : u16 = 0;
        const WORKING : u16 = 1;
        const SOME : u16 = 2;
    } else if #[cfg(target_has_atomic = "32")] {
        type State = AtomicU32;
        const NONE : u32 = 0;
        const WORKING : u32 = 1;
        const SOME : u32 = 2;
    } else if #[cfg(target_has_atomic = "64")] {
        type State = AtomicU64;
        const NONE : u64 = 0;
        const WORKING : u64 = 1;
        const SOME : u64 = 2;
    } else if #[cfg(target_has_atomic = "ptr")] {
        type State = AtomicUsize;
        const NONE : usize = 0;
        const WORKING : usize = 1;
        const SOME : usize = 2;
    } else {
        compile_error!("The current target doen't support atomic operations");
    }
}