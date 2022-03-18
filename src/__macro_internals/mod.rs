mod object_id;
mod once;

use crate::{
    errors::*,
    java_class::{CodegenClass, JavaClass},
    JniRef,
};
use jni::{sys::jobject, JNIEnv};
use nekojni_signatures::Type;
use parking_lot::RwLock;

pub use crate::{globals::set_default_exception_class, panicking::catch_panic_jni};
pub use object_id::IdManager;
pub use once::SignatureCache;

pub use std;

pub mod jni_ref {
    pub use crate::java_class::jni_ref::*;
}

pub trait CreateJniRef: JavaClass + Sized {
    const JAVA_TYPE: Type<'static>;
    fn create_jni_ref(env: &JNIEnv, obj: jobject, is_mut: bool) -> JniRef<Self>;
}

pub trait JavaClassImpl {
    /// Contains the information needed to generate Java or Scala headers for this module.
    const CODEGEN_INFO: Option<CodegenClass> = None;

    /// Called on initialization to register JNI methods.
    fn register_methods(&self, env: JNIEnv) -> Result<()>;

    /// Contains the type that allows access to the JVM components of this class. This is
    /// accessible through the [`JniRef`], though it is normally used through wrapper functions
    /// rather than directly.
    type JvmInterface;

    /// Creates the JvmInterface given a this pointer and the JNI environment.
    fn create_interface(env: &JNIEnv, this: jobject) -> Result<Self::JvmInterface>;
}

pub trait RustContents: JavaClass + Sized + Send + Sync + 'static {
    const ID_FIELD: &'static str;
    fn get_manager() -> &'static IdManager<RwLock<Self>>;
}
