pub mod exports;
pub mod jni_ref;
pub mod object_id;

use crate::{errors::*, java_class::exports::ExportedClass, JniEnv};
use jni::objects::JObject;

// TODO: Generate native-image configurations.

/// A trait representing a Java class.
pub trait JavaClass<'env>: JavaClassImpl<'env> {}

pub trait JavaClassImpl<'env>: Sized + Send + Sync + 'static {
    const INIT_ID: usize;

    const JNI_TYPE: &'static str;
    const JNI_TYPE_SIG: &'static str;

    fn default_ptr() -> &'static Self;

    fn create_jni_ref(
        env: JniEnv<'env>,
        obj: JObject<'env>,
        id: Option<u32>,
    ) -> Result<jni_ref::JniRef<'env, Self>>
    where
        Self: JavaClass<'env>;

    type Cache: Default + 'env;
}

pub trait RustContents<'env>: JavaClass<'env> {
    const ID_FIELD: &'static str;
}

#[derive(Copy, Clone, Debug)]
pub struct JavaClassInfo {
    pub name: &'static str,
    pub exported: &'static Option<ExportedClass>,
}

pub trait JavaModule: JavaModuleImpl {}
pub trait JavaModuleImpl {
    fn get_info(&self) -> &'static [&'static JavaClassInfo];
}
