use crate::*;
use jni::{
    signature::{JavaType, Primitive},
    strings::JNIString,
};

impl<'a> From<ClassName<'a>> for JNIString {
    fn from(name: ClassName<'a>) -> Self {
        JNIString::from(name.display_jni().to_string())
    }
}
impl<'a> From<MethodSig<'a>> for JNIString {
    fn from(name: MethodSig<'a>) -> Self {
        JNIString::from(name.display_jni().to_string())
    }
}
impl<'a> From<Type<'a>> for JNIString {
    fn from(name: Type<'a>) -> Self {
        JNIString::from(name.display_jni().to_string())
    }
}

impl<'a> From<BasicType<'a>> for JavaType {
    fn from(ty: BasicType<'a>) -> Self {
        match ty {
            BasicType::Byte => JavaType::Primitive(Primitive::Byte),
            BasicType::Short => JavaType::Primitive(Primitive::Short),
            BasicType::Int => JavaType::Primitive(Primitive::Int),
            BasicType::Long => JavaType::Primitive(Primitive::Long),
            BasicType::Float => JavaType::Primitive(Primitive::Float),
            BasicType::Double => JavaType::Primitive(Primitive::Double),
            BasicType::Boolean => JavaType::Primitive(Primitive::Boolean),
            BasicType::Char => JavaType::Primitive(Primitive::Char),
            BasicType::Class(clazz) => JavaType::Object(clazz.display_jni().to_string()),
        }
    }
}
impl<'a> From<Type<'a>> for JavaType {
    fn from(ty: Type<'a>) -> Self {
        let mut java_ty: JavaType = ty.basic_sig.into();
        for _ in 0..ty.array_dim {
            java_ty = JavaType::Array(Box::new(java_ty));
        }
        java_ty
    }
}
impl<'a> From<ReturnType<'a>> for JavaType {
    fn from(ty: ReturnType<'a>) -> Self {
        match ty {
            ReturnType::Void => JavaType::Primitive(Primitive::Void),
            ReturnType::Ty(ty) => ty.into(),
        }
    }
}
