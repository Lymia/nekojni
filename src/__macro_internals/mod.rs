mod object_id;
mod once;
mod return_ty;

use crate::{
    errors::*,
    java_class::{CodegenClass, JavaClass},
    JniRef,
};
use jni::{objects::JObject, JNIEnv};
use nekojni_signatures::Type;
use parking_lot::RwLock;

pub use crate::{globals::set_default_exception_class, panicking::catch_panic_jni};
pub use object_id::IdManager;
pub use once::JNIStrCache;
pub use return_ty::ImportReturnTy;

pub use jni;
pub use std;
pub use parking_lot;

pub mod jni_ref {
    pub use crate::java_class::jni_ref::{new_rust, new_wrapped};
}

pub trait JavaClassImpl: Sized + Send + Sync + 'static {
    /// The Java type of this class.
    const JAVA_TYPE: Type<'static>;

    /// Contains the information needed to generate Java or Scala headers for this module.
    const CODEGEN_INFO: Option<CodegenClass> = None;

    /// Called on initialization to register JNI methods.
    fn register_methods(&self, env: JNIEnv) -> Result<()>;

    /// Returns the default pointer for references not generated with [`RustContents`].
    fn default_ptr() -> &'static Self;

    /// Creates a new [`JniRef`] for this class.
    fn create_jni_ref<'env>(env: JNIEnv<'env>, obj: JObject<'env>) -> Result<JniRef<'env, Self>>
    where Self: JavaClass;
}

pub trait RustContents: JavaClass {
    const ID_FIELD: &'static str;
    fn get_manager() -> &'static IdManager<RwLock<Self>>;
}

/// An error function for [`JavaClassImpl::default_ptr`].
#[inline(never)]
pub fn default_ptr_fail() -> ! {
    panic!("internal error: attempted to call `JavaClassImpl::default_ptr` on exported type")
}
