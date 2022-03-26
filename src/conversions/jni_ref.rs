#![allow(deprecated)]

use crate::{conversions::*, java_class::JavaClass, JniRef};
use jni::objects::JObject;

impl<'env, T: JavaClass<'env>> JavaConversion<'env> for JniRef<'env, T> {
    const JAVA_TYPE: Type<'static> = T::JAVA_TYPE;
    type JavaType = JObject<'env>;

    fn to_java(&self, _: JNIEnv<'env>) -> Self::JavaType {
        JniRef::this(self)
    }
    fn to_java_value(&self, _: JNIEnv<'env>) -> JValue<'env> {
        JValue::Object(JniRef::this(self))
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        JObject::from(std::ptr::null_mut())
    }
}
impl<'env, T: JavaClass<'env>> JavaConversionOwned<'env> for JniRef<'env, T> {
    fn from_java(java: Self::JavaType, env: JNIEnv<'env>) -> Self {
        T::create_jni_ref(env, java).unwrap()
    }
    fn from_java_value(java: JValue<'env>, env: JNIEnv<'env>) -> Result<Self> {
        if let JValue::Object(value) = java {
            Ok(Self::from_java(value.into(), env))
        } else {
            jni_bail!(
                "Type error: expected {} got {java:?}",
                T::JAVA_TYPE.display_java()
            );
        }
    }
}
