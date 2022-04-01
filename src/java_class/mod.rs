pub mod exports;
pub mod jni_ref;
pub mod object_id;

use crate::{errors::*, JniEnv};
use jni::objects::JObject;
use nekojni_signatures::Type;
use parking_lot::RwLock;

// TODO: Generate native-image configurations.

/// A trait representing a Java class.
pub trait JavaClass<'env>: JavaClassImpl<'env> {}

pub trait JavaClassImpl<'env>: Sized + Send + Sync + 'static {
    /// The Java type of this class.
    const JAVA_TYPE: Type<'static>;

    /// Contains the information needed to generate Java or Scala headers for this module.
    const CODEGEN_INFO: Option<exports::CodegenClass> = None;

    /// Called on initialization to register JNI methods.
    fn register_methods(&self, env: JniEnv) -> Result<()>;

    /// Returns the default pointer for references not generated with [`RustContents`].
    fn default_ptr() -> &'static Self;

    /// Creates a new [`JniRef`] for this class.
    fn create_jni_ref(env: JniEnv<'env>, obj: JObject<'env>) -> Result<jni_ref::JniRef<'env, Self>>
    where Self: JavaClass<'env>;

    /// The cache type stored in each [`JniRef`].
    type Cache: Default + 'env;
}

pub trait RustContents<'env>: JavaClass<'env> {
    const ID_FIELD: &'static str;
}
