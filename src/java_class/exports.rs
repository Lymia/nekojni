use crate::{errors::*, JniEnv};
use enumset::EnumSet;
use jni::{strings::JNIString, NativeMethod};
use nekojni_utils::{CFlags, FFlags, MFlags};
use std::ffi::c_void;

/// Represents something exported from a Java class defined in Rust.
///
/// This is primarily intended to allow code generation for the Java-side of the Rust bindings.
#[derive(Debug)]
pub enum ExportedItem {
    NativeConstructor {
        flags: EnumSet<MFlags>,
        signature: &'static str,

        native_name: &'static str,
        native_signature: &'static str,

        super_signature: &'static str,
    },
    NativeMethodWrapper {
        flags: EnumSet<MFlags>,
        name: &'static str,
        signature: &'static str,

        native_name: &'static str,
        native_signature: &'static str,

        has_id_param: bool,
    },
    JavaField {
        flags: EnumSet<FFlags>,
        name: &'static str,
        field: &'static str,
    },
}

/// A native method exported from JNI.
#[derive(Debug)]
pub struct RustNativeMethod {
    pub name: &'static str,
    pub sig: &'static str,
    pub fn_ptr: *mut c_void,
    pub is_static: bool,
}
unsafe impl Send for RustNativeMethod {}
unsafe impl Sync for RustNativeMethod {}

pub fn jni_native_name(name: &str, is_static: bool) -> String {
    format!("njni$${}${}", name, if is_static { "s" } else { "m" })
}

/// A trait representing a Java class that may be exported via codegen.
#[derive(Copy, Clone, Debug)]
pub struct ExportedClass {
    pub access: EnumSet<CFlags>,
    pub name: &'static str,
    pub super_class: Option<&'static str>,
    pub implements: &'static [&'static str],

    pub id_field_name: &'static str,
    pub late_init: &'static [&'static str],

    pub exports: &'static [ExportedItem],
    pub native_methods: &'static [RustNativeMethod],
}
impl ExportedClass {
    pub unsafe fn register_natives(&self, env: JniEnv) -> Result<()> {
        let mut methods = Vec::new();
        for method in self.native_methods {
            methods.push(NativeMethod {
                name: JNIString::from(jni_native_name(&method.name, method.is_static)),
                sig: JNIString::from(method.sig),
                fn_ptr: method.fn_ptr,
            });
        }
        env.register_native_methods(self.name, &methods)?;
        Ok(())
    }
}
