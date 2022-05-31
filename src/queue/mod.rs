extern crate alloc;

use core::{sync::atomic::Ordering};
use alloc::collections::VecDeque;
use crate::{waker::Waker, Flag, FALSE, TRUE};
flat_mod!(cell);

pub struct Queue {
    locked: Flag,
    queue: OnceCell<VecDeque<Waker>>
}

impl Queue {
    #[inline(always)]
    pub const fn new () -> Self {
        Self {
            locked: Flag::new(FALSE),
            queue: OnceCell::new()
        }
    }

    #[inline(always)]
    pub fn push (&self, v: Waker) {
        let queue = self.lock();
        queue.push_back(v);
        self.unlock();
    }

    #[inline(always)]
    pub fn wake (&self) {
        let queue = self.lock();
        if let Some(waker) = queue.pop_front() {
            waker.wake()
        }
        self.unlock();
    }

    #[inline(always)]
    fn lock (&self) -> &mut VecDeque<Waker> {
        while self.locked.compare_exchange(FALSE, TRUE, Ordering::AcqRel, Ordering::Acquire).is_err() { core::hint::spin_loop() }
        let queue = self.queue.get_or_default() as *const VecDeque<Waker>;
        unsafe { &mut *(queue as *mut VecDeque<Waker>) }
    }

    #[inline(always)]
    fn unlock (&self) {
        #[cfg(debug_assertions)]
        assert_eq!(TRUE, self.locked.swap(FALSE, Ordering::Release));
        #[cfg(not(debug_assertions))]
        self.locked.store(FALSE, Ordering::Release);
    }
}

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}