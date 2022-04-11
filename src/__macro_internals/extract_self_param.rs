use crate::{conversions::*, java_class::JavaClass, JniEnv, JniRef, JniRefMut};
use jni::sys::jobject;

pub trait ExtractSelfParam<'env> {
    unsafe fn extract(env: JniEnv<'env>, this: jobject) -> Self;
}
impl<'env, T: JavaClass<'env>> ExtractSelfParam<'env> for JniRef<'env, T> {
    unsafe fn extract(env: JniEnv<'env>, this: jobject) -> Self {
        <Self as JavaConversionOwned<'env>>::from_java(this, env)
    }
}
impl<'env, T: JavaClass<'env>> ExtractSelfParam<'env> for JniRefMut<'env, T> {
    unsafe fn extract(env: JniEnv<'env>, this: jobject) -> Self {
        let r = <JniRef<'env, T> as JavaConversionOwned<'env>>::from_java(this, env);
        r.upgrade()
    }
}
