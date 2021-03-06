pub mod exported_class;
pub mod jni_ref;
pub mod object_id;

use crate::{errors::*, java_class::exported_class::ExportedClass, JniEnv};
use jni::objects::JObject;

// TODO: Generate native-image configurations.

/// A trait representing a Java class.
pub trait JavaClass<'env>: JavaClassImpl<'env> {}

pub trait JavaClassType {
    const JNI_TYPE: &'static str;
    const JNI_TYPE_SIG: &'static str;
}
pub trait JavaClassImpl<'env>: JavaClassType + Sized + Send + Sync + 'static {
    const INIT_ID: usize;

    fn default_ptr() -> &'static Self;

    fn create_jni_ref(
        env: JniEnv<'env>,
        obj: JObject<'env>,
        id: Option<u32>,
    ) -> Result<jni_ref::JniRef<'env, Self>>
    where
        Self: JavaClass<'env>;
}

pub trait RustContents<'env>: JavaClass<'env> {
    const ID_FIELD: &'static str;
}

pub trait JavaModule: JavaModuleImpl {}
pub trait JavaModuleImpl {
    fn get_info(&self) -> &'static JavaModuleInfo;
}

#[derive(Copy, Clone, Debug)]
pub struct JavaClassInfo {
    pub name: &'static str,
    pub exported: ExportedClass,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct JavaModuleInfo {
    pub magic: u32,
    pub major_version: usize,
    pub marker_len: usize,
    pub marker_ptr: *const u8,
    pub crate_name: &'static str,
    pub crate_version: &'static str,
    pub init_class_name: &'static str,
    pub except_class_name: &'static str,
    pub class_info: &'static [&'static JavaClassInfo],
}
impl JavaModuleInfo {
    pub unsafe fn get_marker_ptr(&self) -> &'static str {
        let str: &'static [u8] = std::slice::from_raw_parts(self.marker_ptr, self.marker_len);
        std::str::from_utf8_unchecked(str)
    }
}
unsafe impl Send for JavaModuleInfo {}
unsafe impl Sync for JavaModuleInfo {}
