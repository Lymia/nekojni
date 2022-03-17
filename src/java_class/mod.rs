mod exports;
pub(crate) mod jni_ref;

pub use exports::*;
pub use jni_ref::JniRef;

use crate::errors::*;
use jni::{sys::jobject, JNIEnv};

/// A trait representing a Java class.
pub trait JavaClass {
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
