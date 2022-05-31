extern crate alloc;
use alloc::sync::Arc;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        use crate::{Flag, TRUE};

        pub enum Waker {
            Async (core::task::Waker),
            Sync (Arc<Flag>)
        }

        impl Waker {
            #[inline(always)]
            pub fn wake (self) {
                match self {
                    Self::Async (w) => w.wake(),
                    Self::Sync (f) => f.store(TRUE, core::sync::atomic::Ordering::Release)
                }
            }
        }

        impl From<core::task::Waker> for Waker {
            #[inline(always)]
            fn from(x: core::task::Waker) -> Self {
                Self::Async(x)
            }
        }
    } else {
        pub type Waker = core::task::Waker;
    }
}