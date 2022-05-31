use core::{mem::MaybeUninit, sync::atomic::*, cell::UnsafeCell};
use crate::{Flag, FALSE, TRUE};

pub struct OnceCell<T> {
    init: State,
    value: UnsafeCell<MaybeUninit<T>>
}

impl<T> OnceCell<T> {
    #[inline(always)]
    pub const fn new () -> Self {
        Self {
            init: State::new(UNINIT),
            value: UnsafeCell::new(MaybeUninit::uninit())
        }
    }

    #[inline(always)]
    pub const fn new_init (v: T) -> Self {
        Self {
            init: State::new(INIT),
            value: UnsafeCell::new(MaybeUninit::new(v))
        }
    }

    #[inline]
    pub fn get_or_set<F: FnOnce() -> T> (&self, f: F) -> &T {
        match self.init.compare_exchange(UNINIT, WORKING, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) =>  unsafe { 
                (&mut *self.value.get()).write(f()); 
                self.init.store(INIT, Ordering::Release);
            },
            Err(WORKING) => while self.init.load(Ordering::Acquire) == WORKING { core::hint::spin_loop() },
            Err(_) => {}
        }

        unsafe { (&*self.value.get()).assume_init_ref() }
    }

    #[inline(always)]
    pub fn get_or_default (&self) -> &T where T: Default {
        self.get_or_set(Default::default)
    }

    #[inline(always)]
    pub fn try_set (&self, v: T) -> Result<(), &T> {
        match self.init.compare_exchange(UNINIT, WORKING, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => unsafe {
                (&mut *self.value.get()).write(v);
                self.init.store(INIT, Ordering::Release);
                return Ok(());
            },

            Err(WORKING) => while self.init.load(Ordering::Acquire) == WORKING { core::hint::spin_loop() }
            Err(_) => {}
        }
        
        unsafe { Err((&*self.value.get()).assume_init_ref()) }
    }

    #[inline(always)]
    pub fn try_get (&self) -> Option<&T> {
        match self.init.load(Ordering::Acquire) {
            UNINIT => return None,
            WORKING => while self.init.load(Ordering::Acquire) == WORKING { core::hint::spin_loop() },
            _ => {}
        }
        
        unsafe { Some((&*self.value.get()).assume_init_ref()) }
    }
}

unsafe impl<T: Send> Send for OnceCell<T> {}
unsafe impl<T: Sync> Sync for OnceCell<T> {}

pub struct AtomicCell<T> {
    locked: Flag,
    value: UnsafeCell<Option<T>>
}

impl<T> AtomicCell<T> {
    #[inline(always)]
    pub const fn new (v: T) -> Self {
        Self { 
            locked: Flag::new(FALSE),
            value: UnsafeCell::new(Some(v))
        }
    }

    #[inline(always)]
    pub fn try_take (&self) -> Option<T> {
        self.wait_lock();
        let result = unsafe { core::mem::take(&mut *self.value.get()) };
        self.unlock();
        result
    }

    #[inline(always)]
    pub fn try_write (&self, v: T) -> Result<(), T> {
        self.wait_lock();

        let result;
        let value = unsafe { &mut *self.value.get() };

        if value.is_none() { 
            *value = Some(v);
            result = Ok(())
        } else {
            result = Err(v)
        }
        
        self.unlock();
        result
    }

    #[inline(always)]
    fn try_lock (&self) -> bool {
        self.locked.compare_exchange(FALSE, TRUE, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }

    #[inline(always)]
    fn wait_lock (&self) {
        while !self.try_lock() { core::hint::spin_loop() }
    }

    #[inline(always)]
    fn unlock (&self) {
        #[cfg(debug_assertions)]
        assert_eq!(self.locked.swap(FALSE, Ordering::Release), TRUE);
        #[cfg(not(debug_assertions))]
        self.locked.store(FALSE, Ordering::Release)
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_has_atomic = "8")] {
        type State = AtomicU8;
        const UNINIT : u8 = 0;
        const WORKING : u8 = 1;
        const INIT : u8 = 2;
    } else if #[cfg(target_has_atomic = "16")] {
        type State = AtomicU16;
        const UNINIT : u16 = 0;
        const WORKING : u16 = 1;
        const INIT : u16 = 2;
    } else if #[cfg(target_has_atomic = "32")] {
        type State = AtomicU32;
        const UNINIT : u32 = 0;
        const WORKING : u32 = 1;
        const INIT : u32 = 2;
    } else if #[cfg(target_has_atomic = "64")] {
        type State = AtomicU64;
        const UNINIT : u64 = 0;
        const WORKING : u64 = 1;
        const INIT : u64 = 2;
    } else if #[cfg(target_has_atomic = "ptr")] {
        type State = AtomicUsize;
        const UNINIT : usize = 0;
        const WORKING : usize = 1;
        const INIT : usize = 2;
    } else {
        compile_error!("The current target doen't support atomic operations");
    }
}