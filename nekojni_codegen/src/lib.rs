#![deny(unused_must_use)]

mod native_init;
mod native_shutdown;
mod utils;

pub use native_init::*;
pub use native_shutdown::*;
