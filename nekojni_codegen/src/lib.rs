#![deny(unused_must_use)]

extern crate core;

mod classfile;
mod generate_precompiled;
pub mod signatures;

pub use classfile::{CFlags, FFlags, MFlags};
pub use generate_precompiled::*;
