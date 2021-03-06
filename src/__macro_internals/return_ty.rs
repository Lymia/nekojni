use crate::{
    conversions::JavaConversionOwned, errors::*, jni_env::JniEnv, objects::JavaClass, JniRef,
};
use jni::objects::JValue;

pub trait ImportReturnTy<'env> {
    fn from_return_ty(from: &str, env: JniEnv<'env>, value: Result<JValue<'env>>) -> Self;
    const JNI_TYPE: &'static str;
}

impl<'env> ImportReturnTy<'env> for () {
    fn from_return_ty(from: &str, _: JniEnv<'env>, value: Result<JValue<'env>>) -> Self {
        match value {
            Ok(JValue::Void) => (),
            Ok(v) => panic!("method {from} returned error: received {v:?} instead of void"),
            Err(e) => panic!("method {from} returned error: {e}"),
        }
    }
    const JNI_TYPE: &'static str = "V";
}
impl<'env, T: JavaConversionOwned<'env>> ImportReturnTy<'env> for T {
    fn from_return_ty(from: &str, env: JniEnv<'env>, value: Result<JValue<'env>>) -> Self {
        match value {
            Ok(v) => match T::from_java_value(v, env) {
                Ok(v) => v,
                Err(e) => panic!("method {from} returned error: internal type mismatch: {e}"),
            },
            Err(e) => panic!("method {from} returned error: {e}"),
        }
    }
    const JNI_TYPE: &'static str = T::JNI_TYPE;
}
impl<'env, T: ImportReturnTy<'env>> ImportReturnTy<'env> for Result<T> {
    fn from_return_ty(from: &str, env: JniEnv<'env>, value: Result<JValue<'env>>) -> Self {
        Ok(T::from_return_ty(from, env, Ok(value?)))
    }
    const JNI_TYPE: &'static str = T::JNI_TYPE;
}

pub trait ImportCtorReturnTy<'env, T: JavaClass<'env>> {
    fn from_return_ty(from: &str, env: JniEnv<'env>, value: Result<JValue<'env>>) -> Self;
}
impl<'env, T: JavaClass<'env>> ImportCtorReturnTy<'env, T> for JniRef<'env, T> {
    fn from_return_ty(from: &str, env: JniEnv<'env>, value: Result<JValue<'env>>) -> Self {
        match value {
            Ok(v) => match Self::from_java_value(v, env) {
                Ok(v) => v,
                Err(e) => panic!("method {from} returned error: internal type mismatch: {e}"),
            },
            Err(e) => panic!("method {from} returned error: {e}"),
        }
    }
}
impl<'env, T: JavaClass<'env>> ImportCtorReturnTy<'env, T> for Result<JniRef<'env, T>> {
    fn from_return_ty(_: &str, env: JniEnv<'env>, value: Result<JValue<'env>>) -> Self {
        JniRef::<'env, T>::from_java_value(value?, env)
    }
}
