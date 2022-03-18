use crate::errors::*;
use jni::{objects::JValue, sys::*, JNIEnv};
use nekojni_signatures::*;

macro_rules! impl_borrowed_from_owned {
    () => {
        fn from_java_ref<R>(
            java: Self::JavaType,
            env: &JNIEnv,
            func: impl FnOnce(&Self) -> R,
        ) -> R {
            func(&Self::from_java(java, env))
        }
        fn from_java_mut<R>(
            java: Self::JavaType,
            env: &JNIEnv,
            func: impl FnOnce(&mut Self) -> R,
        ) -> R {
            func(&mut Self::from_java(java, env))
        }
    };
}

mod basic;
mod jni_ref;

/// Main trait that converts between Java and Rust types.
pub trait JavaConversion {
    /// The Java type used for this Rust object.
    const JAVA_TYPE: Type<'static>;

    /// The type used for the exported function signature.
    type JavaType;

    /// Convert the Rust type into a Java type.
    fn to_java(&self, env: &JNIEnv) -> Self::JavaType;

    /// Convert the Rust type into a Java method parameter.
    fn to_java_value(&self, env: &JNIEnv) -> JValue;

    /// Convert the Java type into an borrowed Rust type.
    fn from_java_ref<R>(java: Self::JavaType, env: &JNIEnv, func: impl FnOnce(&Self) -> R) -> R;

    /// Convert the Java type into an mutably borrowed Rust type.
    fn from_java_mut<R>(java: Self::JavaType, env: &JNIEnv, func: impl FnOnce(&mut Self) -> R)
        -> R;

    /// Returns the closest thing to a null value in this type. Used as a return type for an JNI
    /// function after returning an exception.
    fn null() -> Self::JavaType;
}

/// Trait that allows converting Java types into owned Rust types.
pub trait JavaConversionOwned: JavaConversion + Sized {
    /// Convert the Java type into an owned Rust type.
    fn from_java(java: Self::JavaType, env: &JNIEnv) -> Self;

    /// Convert the Java return value into an owned Rust type.
    fn from_java_value(java: JValue, env: &JNIEnv) -> Result<Self>;
}
