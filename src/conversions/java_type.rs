use crate::conversions::java_type::sealed::Sealed;
use jni::sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use std::any::Any;

mod sealed {
    pub trait Sealed {}
}

/// Marker trait for Java types that can be used through the JNI interface.
///
/// An exhaustive list of possible types:
/// * [`jboolean`]
/// * [`jbyte`]
/// * [`jchar`]
/// * [`jdouble`]
/// * [`jfloat`]
/// * [`jint`]
/// * [`jlong`]
/// * [`jobject`]
/// * [`jshort`]
pub trait JniAbiType: Sealed + Any {}

macro_rules! simple_type {
    ($($ty:ty,)*) => {$(
        impl Sealed for $ty {}
        impl JniAbiType for $ty {}
    )*};
}
simple_type! {
    (), jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort,
}
