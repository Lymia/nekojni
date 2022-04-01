use crate::{errors::*, jni_env::JniEnv};
use jni::{objects::JValue, sys::*};
use nekojni_signatures::*;

macro_rules! impl_borrowed_from_owned {
    ($env:lifetime) => {
        fn from_java_ref<R>(
            java: Self::JavaType,
            env: JniEnv<$env>,
            func: impl FnOnce(&Self) -> R,
        ) -> R {
            func(&Self::from_java(java, env))
        }
        fn from_java_mut<R>(
            java: Self::JavaType,
            env: JniEnv<$env>,
            func: impl FnOnce(&mut Self) -> R,
        ) -> R {
            func(&mut Self::from_java(java, env))
        }
    };
}

mod basic;
mod jni_ref;
mod strings;

/// Main trait that converts between Java and Rust types.
pub trait JavaConversion<'env> {
    /// The Java type used for this Rust object.
    const JAVA_TYPE: Type<'static>;

    /// The type used for the exported function signature.
    type JavaType;

    /// Convert the Rust type into a Java type.
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType;

    /// Convert the Rust type into a Java method parameter.
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env>;

    /// Convert the Java type into an borrowed Rust type.
    fn from_java_ref<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&Self) -> R,
    ) -> R;

    /// Convert the Java type into an mutably borrowed Rust type.
    fn from_java_mut<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&mut Self) -> R,
    ) -> R;

    /// Returns the closest thing to a null value in this type. Used as a return type for an JNI
    /// function after returning an exception.
    fn null() -> Self::JavaType;
}

/// Trait that allows converting Java types into owned Rust types.
pub trait JavaConversionOwned<'env>: JavaConversion<'env> + Sized {
    /// Convert the Java type into an owned Rust type.
    fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self;

    /// Convert the Java return value into an owned Rust type.
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self>;
}

/// Trait that converts Java types into Java return values.
pub trait JavaReturnConversion<'env> {
    /// The Java type used for this Rust object.
    const JAVA_RETURN_TYPE: ReturnType<'env>;

    /// The type used for the exported function signature.
    type JavaRetType;

    /// Convert the Rust type into a Java type.
    fn to_java_ret(&self, env: JniEnv<'env>) -> Self::JavaRetType;

    /// Returns the closest thing to a null value in this type. Used as a return type for an JNI
    /// function after returning an exception.
    fn null_ret() -> Self::JavaRetType;
}

impl<'env> JavaReturnConversion<'env> for () {
    const JAVA_RETURN_TYPE: ReturnType<'env> = ReturnType::Void;
    type JavaRetType = ();
    fn to_java_ret(&self, _: JniEnv<'env>) -> Self::JavaRetType {}
    fn null_ret() -> Self::JavaRetType {}
}
impl<'env, T: JavaConversion<'env>> JavaReturnConversion<'env> for T {
    const JAVA_RETURN_TYPE: ReturnType<'env> = ReturnType::Ty(T::JAVA_TYPE);
    type JavaRetType = T::JavaType;
    fn to_java_ret(&self, env: JniEnv<'env>) -> Self::JavaRetType {
        T::to_java(self, env)
    }
    fn null_ret() -> Self::JavaRetType {
        T::null()
    }
}
