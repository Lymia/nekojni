#![allow(deprecated)]

use crate::{__macro_internals::CreateJniRef, conversions::*, JniRef};
use jni::objects::JObject;

impl<T: CreateJniRef> JavaConversion for JniRef<T> {
    const JAVA_TYPE: Type<'static> = T::JAVA_TYPE;
    type JavaType = jobject;

    fn to_java(&self, _: &JNIEnv) -> Self::JavaType {
        JniRef::this(self)
    }
    fn to_java_value(&self, env: &JNIEnv) -> JValue {
        JValue::Object(JObject::from(self.to_java(env)))
    }

    fn from_java_ref<R>(java: Self::JavaType, env: &JNIEnv, func: impl FnOnce(&Self) -> R) -> R {
        let this = T::create_jni_ref(env, java, false);
        func(&this)
    }
    fn from_java_mut<R>(
        java: Self::JavaType,
        env: &JNIEnv,
        func: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let mut this = T::create_jni_ref(env, java, true);
        func(&mut this)
    }

    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
