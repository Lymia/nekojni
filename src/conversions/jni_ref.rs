use super::*;
use crate::{
    java_class::{jni_ref::JniRefType, JavaClass, JavaClassType},
    JniRef, JniRefMut,
};
use jni::objects::JObject;

impl<'env, T: JavaClassType, R: JniRefType> JavaConversionType for JniRef<'env, T, R> {
    type JavaType = jobject;
    const JNI_TYPE: &'static str = T::JNI_TYPE_SIG;
}

unsafe impl<'env, T: JavaClass<'env>> JavaConversion<'env> for JniRef<'env, T> {
    fn to_java(&self, _: JniEnv<'env>) -> Self::JavaType {
        JniRef::this(self).into_inner()
    }
    fn to_java_value(&self, _: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JniRef::this(self))
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
unsafe impl<'env, T: JavaClass<'env>> JavaConversionOwned<'env> for JniRef<'env, T> {
    unsafe fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self {
        T::create_jni_ref(env, JObject::from(java), None).unwrap()
    }
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
        Ok(unsafe { Self::from_java(java.l()?.into_inner(), env) })
    }
}

unsafe impl<'env, T: JavaClass<'env>> JavaConversion<'env> for JniRefMut<'env, T> {
    fn to_java(&self, _: JniEnv<'env>) -> Self::JavaType {
        JniRef::this(self).into_inner()
    }
    fn to_java_value(&self, _: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JniRef::this(self))
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
unsafe impl<'env, T: JavaClass<'env>> JavaConversionOwned<'env> for JniRefMut<'env, T> {
    unsafe fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self {
        T::create_jni_ref(env, JObject::from(java), None)
            .unwrap()
            .upgrade_ref()
    }
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
        Ok(unsafe { Self::from_java(java.l()?.into_inner(), env) })
    }
}
