#![feature(backtrace, downcast_unchecked)]
#![deny(unused_must_use)]

#[macro_use]
mod errors;

/// The module for nekojni's internal types. This is in no way public API!!
#[deprecated = "This module is for internal use by nekojni's macros, and should not be used by \
                external code. There are no API guarantees!"]
#[doc(hidden)]
pub mod __macro_internals;

mod internal;
mod java_class;
mod jni_env;

pub use errors::{Error, Result};
pub use java_class::jni_ref::{JniRef, JniRefMut};
pub use jni_env::JniEnv;

/// The module containing the types used for conversions between Java and Rust types.
pub mod conversions;

/// The module containing types used to represent Java objects.
pub mod objects {
    pub use crate::java_class::JavaClass;
}

#[doc(inline)]
/// The module containing types that represent Java type signatures.
pub use nekojni_signatures as signatures;

#[doc(inline)]
pub use nekojni_macros::{jni, jni_export};

/// Generates types essential for the attributes of `nekojni` to work as intended.
///
/// This should be used in the root of your crate, and will result in an error if it is used
/// anywhere else.
///
/// The first parameter is the name of the module type, and the second parameter is the name
/// of the class it exports into Java. This class will mostly be used to contain an initialization
/// function that will be called from the Java-side of other types exported by this module.
///
/// ## Examples
///
/// ```rust
/// use nekojni::jni_module;
/// jni_module!(pub FooModule, "moe.lymia.FooInit");
/// ```
#[macro_export]
macro_rules! jni_module {
    (
        $module_vis:vis $module_name:ident, $init_class_name:expr $(,)?
    ) => {
        /// The JNI module exported by this crate.
        $module_vis struct $module_name;

        /// The module used by nekojni's codegen for this crate.
        #[deprecated = "This module is for internal use by nekojni! It is not public API for \
                        either the crate it is defined in or its users."]
        #[doc(hidden)]
        #[allow(deprecated)]
        mod nekojni__jni_module {
            use $crate::{jni_export, jni};
            use $crate::__macro_internals::*;

            #[allow(unused)]
            mod check_path {
                struct CheckPathStruct;
                fn check_is_macro_called_in_crate_root() {
                    let val = crate::nekojni__jni_module::check_path::CheckPathStruct;
                    ::std::mem::drop(val);
                }
            }

            pub enum InitHelper { }

            #[jni_export]
            #[jni(java_path = $init_class_name)]
            /// A class used to help initialize this library.
            impl InitHelper {
                /// Initializes this library. This requires that the native library has already
                /// been loaded beforehand using `System.loadLibrary`.
                ///
                /// This is automatically called from the static initializer of all functions in
                /// the library with native functions, and hence should never need to be directly
                /// called.
                pub fn initialize(ctx: $crate::JniEnv) {

                }
            }
        }
    };
}
