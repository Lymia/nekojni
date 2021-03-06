use crate::{java_class::JavaClass, JniEnv, JniRef, JniRefMut};
use jni::{objects::JObject, sys::jobject};

pub trait ExtractSelfParam<'env> {
    unsafe fn extract(env: JniEnv<'env>, this: jobject, id: Option<u32>) -> Self;
}
impl<'env, T: JavaClass<'env>> ExtractSelfParam<'env> for JniRef<'env, T> {
    unsafe fn extract(env: JniEnv<'env>, this: jobject, id: Option<u32>) -> Self {
        T::create_jni_ref(env, JObject::from(this), id).unwrap()
    }
}
impl<'env, T: JavaClass<'env>> ExtractSelfParam<'env> for JniRefMut<'env, T> {
    unsafe fn extract(env: JniEnv<'env>, this: jobject, id: Option<u32>) -> Self {
        let r = <JniRef<'env, T> as ExtractSelfParam<'env>>::extract(env, this, id);
        r.upgrade_ref()
    }
}
