use crate::{conversions::JavaConversionOwned, errors::*};
use jni::{objects::JValue, JNIEnv};

pub trait ImportReturnTy<'env> {
    fn from_return_ty(from: &str, env: JNIEnv<'env>, value: Result<JValue<'env>>) -> Self;
}

impl<'env, T: JavaConversionOwned<'env>> ImportReturnTy<'env> for T {
    fn from_return_ty(from: &str, env: JNIEnv<'env>, value: Result<JValue<'env>>) -> Self {
        match value {
            Ok(v) => match T::from_java_value(v, env) {
                Ok(v) => v,
                Err(e) => panic!("method {from} returned error: internal type mismatch: {e}"),
            },
            Err(e) => panic!("method {from} returned error: {e}"),
        }
    }
}
impl<'env, T: JavaConversionOwned<'env>> ImportReturnTy<'env> for Result<T> {
    fn from_return_ty(_: &str, env: JNIEnv<'env>, value: Result<JValue<'env>>) -> Self {
        T::from_java_value(value?, env)
    }
}
