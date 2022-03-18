use crate::{conversions::JavaConversionOwned, errors::*};
use jni::{objects::JValue, JNIEnv};
use nekojni_signatures::ReturnType;

pub trait ImportReturnTy<'env> {
    fn from_return_ty(from: &str, env: JNIEnv<'env>, value: Result<JValue<'env>>) -> Self;
    const JAVA_TYPE: ReturnType<'static>;
}

impl<'env> ImportReturnTy<'env> for () {
    fn from_return_ty(from: &str, _: JNIEnv<'env>, value: Result<JValue<'env>>) -> Self {
        match value {
            Ok(JValue::Void) => (),
            Ok(v) => panic!("method {from} returned error: received {v:?} instead of void"),
            Err(e) => panic!("method {from} returned error: {e}"),
        }
    }
    const JAVA_TYPE: ReturnType<'static> = ReturnType::Void;
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
    const JAVA_TYPE: ReturnType<'static> = ReturnType::Ty(T::JAVA_TYPE);
}
impl<'env, T: JavaConversionOwned<'env>> ImportReturnTy<'env> for Result<T> {
    fn from_return_ty(_: &str, env: JNIEnv<'env>, value: Result<JValue<'env>>) -> Self {
        T::from_java_value(value?, env)
    }
    const JAVA_TYPE: ReturnType<'static> = ReturnType::Ty(T::JAVA_TYPE);
}
