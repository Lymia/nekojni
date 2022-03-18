use crate::{conversions::JavaConversionOwned, errors::*, java_class::JavaClass};
use jni::{objects::JValue, JNIEnv};

pub trait ImportReturnTy<'a> {
    fn from_return_ty(from: &str, env: &'a JNIEnv, value: Result<JValue<'a>>) -> Self;
}

impl<'a, T: JavaConversionOwned> ImportReturnTy<'a> for T {
    fn from_return_ty(from: &str, env: &'a JNIEnv, value: Result<JValue<'a>>) -> Self {
        match value {
            Ok(v) => match T::from_java_value(v, env) {
                Ok(v) => v,
                Err(e) => panic!("method {from} returned error: internal type mismatch: {e}"),
            },
            Err(e) => panic!("method {from} returned error: {e}"),
        }
    }
}
impl<'a, T: JavaConversionOwned> ImportReturnTy<'a> for Result<T> {
    fn from_return_ty(_: &str, env: &'a JNIEnv, value: Result<JValue<'a>>) -> Self {
        T::from_java_value(value?, env)
    }
}
