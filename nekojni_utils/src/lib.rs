#![deny(unused_must_use)]

extern crate core;

#[cfg(feature = "classfile")]
mod class_data;
#[cfg(feature = "classfile")]
mod classfile;
pub mod constcat;
mod flags;
mod generate_precompiled;
#[cfg(feature = "hash")]
mod hash_util;
#[cfg(feature = "classfile")]
mod native_class_wrapper;
#[cfg(feature = "signature")]
pub mod signatures;
#[cfg(feature = "classfile")]
mod utils;

#[cfg(feature = "classfile")]
pub use class_data::*;
pub use flags::{CFlags, FFlags, MFlags};
pub use generate_precompiled::generate_shutdown_hook;
#[cfg(feature = "hash")]
pub use hash_util::Hasher;
#[cfg(feature = "classfile")]
pub use native_class_wrapper::NativeClassWrapper;
