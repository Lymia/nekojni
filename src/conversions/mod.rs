mod basic;

use jni::{sys::*, JNIEnv};
use nekojni_signatures::*;

/// Main trait that converts between Java and Rust types.
pub trait JavaConversion {
    /// The Java type used for this Rust object.
    const JAVA_TYPE: Type<'static>;

    /// The type used for the exported function signature.
    type JavaType;

    /// Convert the Rust type into a Java type.
    fn into_java(self, env: &JNIEnv) -> Self::JavaType;

    /// Convert the Java type into a Rust type.
    fn from_java(java: Self::JavaType, env: &JNIEnv) -> Self;

    /// Returns the closest thing to a null value in this type. Used as a return type for an JNI
    /// function after returning an exception.
    fn null() -> Self::JavaType;
}
