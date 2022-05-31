#![no_std]
use core::sync::atomic::*;

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

cfg_if::cfg_if! {
    if #[cfg(target_has_atomic = "8")] {
        pub(crate) type Flag = AtomicBool;
        pub(crate) const TRUE : bool = true;
        pub(crate) const FALSE : bool = false;
    } else if #[cfg(target_has_atomic = "16")] {
        pub(crate) type Flag = AtomicU16;
        pub(crate) const TRUE : u16 = 1;
        pub(crate) const FALSE : u16 = 0;
    } else if #[cfg(target_has_atomic = "32")] {
        pub(crate) type Flag = AtomicU32;
        pub(crate) const TRUE : u32 = 1;
        pub(crate) const FALSE : u32 = 0;
    } else if #[cfg(target_has_atomic = "64")] {
        pub(crate) type Flag = AtomicU64;
        pub(crate) const TRUE : u64 = 1;
        pub(crate) const FALSE : u64 = 0;
    } else if #[cfg(target_has_atomic = "ptr")] {
        pub(crate) type Flag = AtomicUsize;
        pub(crate) const TRUE : usize = 1;
        pub(crate) const FALSE : usize = 0;
    } else {
        compile_error!("The current target doen't support atomic operations");
    }
}

flat_mod!(regular);
pub mod movable;
pub mod guards;

pub(crate) mod waker;
pub(crate) mod queue;