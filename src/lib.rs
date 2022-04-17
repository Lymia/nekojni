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
    pub use crate::java_class::{JavaClass, JavaModule};
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
        mod __njni_module_info {
            use $crate::{jni_export, jni, Result};
            use $crate::__macro_internals::*;

            #[allow(unused)]
            mod check_path {
                struct CheckPathStruct;
                fn check_is_macro_called_in_crate_root() {
                    let val = crate::__njni_module_info::check_path::CheckPathStruct;
                    ::std::mem::drop(val);
                }
            }

            pub enum InitHelper { }

            #[jni_export_internal]
            #[jni(java_path = $init_class_name)]
            /// A class used to help initialize this library.
            impl InitHelper {
                /// Initializes this library. This requires that the native library has already
                /// been loaded beforehand using `System.loadLibrary`.
                ///
                /// This is automatically called from the static initializer of all functions in
                /// the library with native functions, and hence should never need to be directly
                /// called.
                #[jni(__njni_direct_export = $init_class_name)]
                pub fn initialize(env: $crate::JniEnv) -> Result<()> {
                    let info = crate::$module_name.get_info();

                    for class in info {
                        if let Some(exported) = &class.exported {
                            exported.register_natives(env)?;
                        }
                    }

                    Ok(())
                }

                /// A method exported from the .so/.dylib/.dll to allow the cli tool to pull
                /// information from the binary.
                #[jni(__njni_export_module_info = $init_class_name)]
                pub extern "C" fn __njni_module_info()
                    -> (&'static str, &'static [&'static JavaClassInfo])
                {
                    (MARKER_STR, crate::$module_name.get_info())
                }
            }

            pub struct GatherClasses<'a>(
                pub std::cell::RefCell<&'a mut Vec<&'static JavaClassInfo>>,
            );

            const CL_ID: usize = <InitHelper as JavaClassImpl>::INIT_ID;
            impl $crate::objects::JavaModule for crate::$module_name { }
            impl JavaModuleImpl for crate::$module_name {
                #[inline(never)]
                fn get_info(&self) -> &'static [&'static JavaClassInfo] {
                    static CACHE: OnceCache<Vec<&'static JavaClassInfo>> = OnceCache::new();
                    &CACHE.init(get_info_raw)
                }
            }

            fn get_info_raw() -> Vec<&'static JavaClassInfo> {
                let mut classes = Vec::new();
                let classes_obj = GatherClasses(std::cell::RefCell::new(&mut classes));

                let helper = DerefRamp::<CL_ID, _>(&classes_obj);
                (&helper).run_chain_fwd();
                let helper = DerefRamp::<{CL_ID - 1}, _>(&classes_obj);
                (&helper).run_chain_rev();

                classes
            }
        }
    };
}
