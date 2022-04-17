use crate::{errors::*, JniEnv};
use enumset::EnumSet;
use jni::{strings::JNIString, NativeMethod};
use nekojni_classfile::{CFlags, FFlags, MFlags};
use std::ffi::c_void;

/// Represents something exported from a Java class defined in Rust.
///
/// This is primarily intended to allow code generation for the Java-side of the Rust bindings.
#[derive(Debug)]
#[non_exhaustive]
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

fn jni_native_name(name: &str, is_static: bool) -> String {
    format!("njni$${}${}", name, if is_static { "s" } else { "m" })
}

/// A trait representing a Java class that may be exported via codegen.
#[derive(Debug)]
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
    pub fn register_natives(&self, env: JniEnv) -> Result<()> {
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

    /*
    pub fn generate_class(&self) -> Vec<(String, Vec<u8>)> {
        let mut class = ClassExporter::new(
            self.access,
            &self.name,
            match &self.super_class {
                None => {
                    static OBJECT_CN: ClassName<'static> =
                        ClassName::new(&["java", "lang"], "String");
                    &OBJECT_CN
                }
                Some(v) => v,
            },
            self.id_field_name,
        );

        for exports in self.exports {
            match exports {
                ExportedItem::NativeConstructor {
                    flags,
                    signature,
                    native_name,
                    native_signature,
                    super_signature,
                } => {
                    class.export_constructor(
                        *flags,
                        &signature,
                        &jni_native_name(native_name, true),
                        &native_signature,
                        &super_signature,
                        self.late_init,
                    );
                }
                ExportedItem::NativeMethodWrapper {
                    flags,
                    name,
                    signature,
                    native_name,
                    has_id_param,
                } => {
                    let mut params = Vec::new();
                    if *has_id_param {
                        params.push(Type::Int);
                    }
                    for param in signature.params.as_slice() {
                        params.push(param.clone());
                    }
                    let native_sig = MethodSig {
                        ret_ty: signature.ret_ty.clone(),
                        params: StaticList::Borrowed(&params),
                    };

                    class.export_native_wrapper(
                        *flags,
                        name,
                        &signature,
                        &jni_native_name(native_name, flags.contains(MFlags::Static)),
                        &native_sig,
                        *has_id_param,
                    );
                }
                ExportedItem::JavaField { flags, name, field } => {
                    class.export_field(*flags, name, &field);
                }
            }
        }
        for method in self.native_methods {
            class.export_native(method.name, &method.sig, method.is_static);
        }

        class.into_vec()
    }
    */
}
