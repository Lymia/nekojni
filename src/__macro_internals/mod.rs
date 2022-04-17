mod extract_self_param;
mod once;
mod registration;
mod return_ty;

pub use crate::{
    internal::{
        globals::set_default_exception_class,
        panicking::{catch_panic_jni, MethodReturn},
    },
    java_class::{exports, JavaClassImpl, JavaClassInfo, JavaModuleImpl, RustContents},
};
pub use constcat::constcat;
pub use extract_self_param::*;
pub use nekojni_classfile::{CFlags, FFlags, MFlags};
pub use nekojni_macros::jni_export_internal;
pub use once::OnceCache;
pub use registration::*;
pub use return_ty::ImportReturnTy;

pub use enumset;
pub use jni;
pub use parking_lot;
pub use std;

use crate::{
    java_class::{jni_ref::JniRefType, JavaClass},
    jni_env::JniEnv,
    JniRef,
};

pub mod jni_ref {
    pub use crate::java_class::jni_ref::{new_rust, new_wrapped};

    use crate::{
        java_class::{jni_ref::JniRefType, JavaClass},
        JniRef,
    };

    pub fn get_cache<'a, 'env, T: JavaClass<'env>, R: JniRefType>(
        r: &'a JniRef<'env, T, R>,
    ) -> &'a T::Cache {
        &r.cache
    }
}

/// An error function for [`JavaClassImpl::default_ptr`].
#[inline(never)]
pub fn default_ptr_fail() -> ! {
    panic!("internal error: attempted to call `JavaClassImpl::default_ptr` on exported type")
}

/// Function for typechecking [`JniRef`]s
pub fn check_jniref<'env, T: JavaClass<'env>, R: JniRefType>(_: JniRef<'env, T, R>) {
    unreachable!()
}

/// Function for typechecking [`JniEnv`]s
pub fn check_jnienv(_: JniEnv) {
    unreachable!()
}

/// Helper function for typechecking.
#[inline(never)]
pub fn promise<T>() -> T {
    unreachable!()
}

/// A version string to allow for detecting binaries compiled with an incompatible version of
/// nekojni. Not 100% reliable, but good enough.
pub static MARKER_STR: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"),);
