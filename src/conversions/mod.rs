use crate::{errors::*, jni_env::JniEnv};
use jni::{objects::JValue, sys::*};

macro_rules! impl_borrowed_from_owned {
    ($env:lifetime) => {
        unsafe fn from_java_ref<R>(
            java: Self::JavaType,
            env: JniEnv<$env>,
            func: impl FnOnce(&Self) -> R,
        ) -> R {
            func(&Self::from_java(java, env))
        }
        unsafe fn from_java_mut<R>(
            java: Self::JavaType,
            env: JniEnv<$env>,
            func: impl FnOnce(&mut Self) -> R,
        ) -> R {
            func(&mut Self::from_java(java, env))
        }
    };
}

mod basic;
mod java_type;
mod jni_ref;
mod objects;
mod strings;

pub use java_type::*;

/// Helper type that proves [`JavaConversion`]s with different lifetimes return the same value.
pub trait JavaConversionType {
    /// The type used for the exported function signature.
    type JavaType: JniAbiType;

    /// The JNI type used for this Rust object.
    const JNI_TYPE: &'static str;
}

/// Main trait that converts between Java and Rust types.
///
/// # Safety
///
/// This is marked unsafe as returning raw pointers is inherently unsafe, despite the fact that
/// the `jni` crate allows this to be done without any unsafety. The value of `JavaType` must be
/// from the input [`JniEnv`] or else there will be an use-after-free.
pub unsafe trait JavaConversion<'env>: JavaConversionType {
    /// Convert the Rust type into a Java type.
    ///
    /// # Safety
    ///
    /// The value returned must be valid for the lifetime of [`JniEnv`].
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType;

    /// Convert the Rust type into a Java method parameter.
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env>;

    /// Convert the Java type into an borrowed Rust type.
    ///
    /// # Safety
    ///
    /// The value passed in must be valid for the lifetime of the [`JniEnv`].
    unsafe fn from_java_ref<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&Self) -> R,
    ) -> R;

    /// Convert the Java type into an mutably borrowed Rust type.
    ///
    /// # Safety
    ///
    /// The value passed in must be valid for the lifetime of the [`JniEnv`].
    unsafe fn from_java_mut<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&mut Self) -> R,
    ) -> R;

    /// Returns the closest thing to a null value in this type. Used as a return type for an JNI
    /// function after returning an exception.
    fn null() -> Self::JavaType;
}

/// Trait that allows converting Java types into owned Rust types.
///
/// # Safety
///
/// This is marked unsafe as returning raw pointers is inherently unsafe, despite the fact that
/// the `jni` crate allows this to be done without any unsafety. The value of `JavaType` must be
/// from the input [`JniEnv`] or else there will be an use-after-free.
pub unsafe trait JavaConversionOwned<'env>: JavaConversion<'env> + Sized {
    /// Convert the Java type into an owned Rust type.
    ///
    /// # Safety
    ///
    /// The value passed in must be valid for the lifetime of the [`JniEnv`].
    unsafe fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self;

    /// Convert the Java return value into an owned Rust type.
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self>;
}

/// Trait that converts Java types into Java return values.
///
/// # Safety
///
/// This is marked unsafe as returning raw pointers is inherently unsafe, despite the fact that
/// the `jni` crate allows this to be done without any unsafety. The value of `JavaType` must be
/// from the input [`JniEnv`] or else there will be an use-after-free.
pub unsafe trait JavaReturnConversion<'env>: JavaConversionType {
    /// Convert the Rust type into a Java type.
    fn to_java_ret(&self, env: JniEnv<'env>) -> Self::JavaType;

    /// Returns the closest thing to a null value in this type. Used as a return type for an JNI
    /// function after returning an exception.
    fn null_ret() -> Self::JavaType;
}

impl JavaConversionType for () {
    type JavaType = ();
    const JNI_TYPE: &'static str = "V";
}
unsafe impl<'env> JavaReturnConversion<'env> for () {
    fn to_java_ret(&self, _: JniEnv<'env>) -> Self::JavaType {}
    fn null_ret() -> Self::JavaType {}
}

unsafe impl<'env, T: JavaConversion<'env>> JavaReturnConversion<'env> for T {
    fn to_java_ret(&self, env: JniEnv<'env>) -> Self::JavaType {
        T::to_java(self, env)
    }
    fn null_ret() -> Self::JavaType {
        T::null()
    }
}
