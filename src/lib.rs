#![feature(backtrace)]
#![deny(unused_must_use)]

#[macro_use]
mod errors;

/// The module for nekojni's internal types. This is in no way public API!!
#[deprecated = "This module is for internal use by nekojni's macros, and should not be used by \
                external code. There are no API guarantees!"]
#[doc(hidden)]
pub mod __macro_internals;

mod conversions;
mod panicking;

pub use conversions::*;
pub use errors::{Error, Result};

#[doc(inline)]
/// The module containing types that represent Java type signatures.
pub use nekojni_signatures as signatures;