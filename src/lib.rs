#![allow(incomplete_features)]
#![deny(unused_must_use, unused_imports)]
#![feature(downcast_unchecked, generic_const_exprs, label_break_value)]

#[macro_use]
mod errors;

mod internal;
mod java_class;
mod jni_env;

pub use errors::{Error, Result};
pub use java_class::jni_ref::{JniRef, JniRefMut};
pub use jni_env::JniEnv;

/// The module containing the types used for conversions between Java and Rust types.
// TODO: Make this private.
pub mod conversions;

/// The module for nekojni's internal types. This is in no way public API!!
#[deprecated = "This module is for internal use by nekojni's macros. There are no API guarantees, \
                and using any types from this module may cause **UNDEFINED BEHAVIOR** in safe \
                code!"]
#[doc(hidden)]
pub mod __macro_internals;

/// The module containing types used to represent Java objects.
pub mod objects {
    pub use crate::{
        java_class::{JavaClass, JavaModule},
        jni_env::objects::*,
    };
}

#[doc(inline)]
pub use nekojni_macros::{jni, jni_export, jni_import};

/// Generates types essential for the attributes of `nekojni` to work as intended.
///
/// This should be used in the root of your crate, and will result in an error if it is used
/// anywhere else.
///
/// The first parameter is the name of the module type. This is only accessible from Rust, and may
/// be used to load this module using the Invocation API.
///
/// The second parameter is the name of the class it exports into Java. This class will mostly be
/// used to contain an initialization function that will be called from the Java-side of other
/// types exported by this module. It is not meant to be used publicly, but is needed for Java to
/// properly link the initialization function.
///
/// The third parameter is the name of the exception class it exports into Java. This class is
/// exported as a `RuntimeException` that Rust panics or errors will be wrapped in.
///
/// ## Examples
///
/// ```rust
/// use nekojni::jni_module;
/// jni_module!(pub FooModule, "moe.lymia.foo.FooInit", "moe.lymia.foo.FooException");
/// ```
#[macro_export]
macro_rules! jni_module {
    (
        $module_vis:vis $module_name:ident, $init_class_name:expr, $except_class_name:expr $(,)?
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

            pub const EXCEPTION_CLASS: &'static str = java_name_to_jni!($except_class_name);

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

                    // load all native methods from all classes
                    for class in info.class_info {
                        unsafe {
                            class.exported.register_natives(env)?;
                        }
                    }

                    Ok(())
                }

                /// A method exported from the .so/.dylib/.dll to allow the cli tool to pull
                /// information from the binary.
                #[jni(__njni_export_module_info = $init_class_name)]
                pub extern "C" fn __njni_module_info() -> &'static JavaModuleInfo {
                    crate::$module_name.get_info()
                }
            }

            pub struct GatherClasses<'a>(
                pub std::cell::RefCell<&'a mut Vec<&'static JavaClassInfo>>,
            );

            const CL_ID: usize = <InitHelper as JavaClassImpl>::INIT_ID;
            impl $crate::objects::JavaModule for crate::$module_name { }
            impl JavaModuleImpl for crate::$module_name {
                #[inline(never)]
                fn get_info(&self) -> &'static JavaModuleInfo {
                    static CACHE: OnceCache<JavaModuleInfo> = OnceCache::new();
                    CACHE.init(|| {
                        static CACHE: OnceCache<Vec<&JavaClassInfo>> = OnceCache::new();
                        let classes = &CACHE.init(get_info_raw);
                        JavaModuleInfo {
                            magic: MAGIC_NUMBER,
                            major_version: MAJOR_VERSION,
                            marker_len: MARKER_STR.len(),
                            marker_ptr: MARKER_STR.as_ptr(),
                            crate_name: env!("CARGO_PKG_NAME"),
                            crate_version: env!("CARGO_PKG_VERSION"),
                            init_class_name: java_name_to_jni!($init_class_name),
                            except_class_name: java_name_to_jni!($except_class_name),
                            class_info: classes,
                        }
                    })
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
