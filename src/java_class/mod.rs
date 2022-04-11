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
    const JAVA_TYPE: Type<'static>;
    const CLASS_INFO: Option<exports::ExportedClass> = None;

    fn register_methods(&self, env: JniEnv) -> Result<()>;

    fn default_ptr() -> &'static Self;

    fn create_jni_ref(env: JniEnv<'env>, obj: JObject<'env>) -> Result<jni_ref::JniRef<'env, Self>>
    where Self: JavaClass<'env>;

    type Cache: Default + 'env;
}

pub trait RustContents<'env>: JavaClass<'env> {
    const ID_FIELD: &'static str;
}
