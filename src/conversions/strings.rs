use super::*;
use crate::internal::panicking::MethodReturn;
use jni::objects::JObject;

impl JavaConversionType for str {
    type JavaType = jobject;
    const JNI_TYPE: &'static str = "Ljava/lang/String;";
}
unsafe impl<'env> JavaConversion<'env> for str {
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType {
        (*env.new_string(self).unwrap()).into_inner()
    }
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    unsafe fn from_java_ref<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&Self) -> R,
    ) -> R {
        let str = String::from_java(java, env);
        func(&str)
    }
    unsafe fn from_java_mut<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let mut str = String::from_java(java, env);
        func(&mut str)
    }
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}

impl JavaConversionType for String {
    type JavaType = jobject;
    const JNI_TYPE: &'static str = "Ljava/lang/String;";
}
unsafe impl<'env> JavaConversion<'env> for String {
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType {
        str::to_java(self.as_str(), env)
    }
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
unsafe impl<'env> JavaConversionOwned<'env> for String {
    unsafe fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self {
        env.get_string(java.into()).unwrap().into()
    }
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
        Ok(unsafe { Self::from_java(java.l()?.into_inner(), env) })
    }
}

impl JavaConversionType for [u8] {
    type JavaType = jobject;
    const JNI_TYPE: &'static str = "[B";
}
unsafe impl<'env> JavaConversion<'env> for [u8] {
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType {
        env.byte_array_from_slice(&self).unwrap()
    }
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    unsafe fn from_java_ref<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&Self) -> R,
    ) -> R {
        let vec = Vec::<u8>::from_java(java, env);
        func(&vec)
    }
    unsafe fn from_java_mut<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let mut vec = Vec::<u8>::from_java(java, env);
        func(&mut vec)
    }
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}

impl JavaConversionType for Vec<u8> {
    type JavaType = jobject;
    const JNI_TYPE: &'static str = "[B";
}
unsafe impl<'env> JavaConversion<'env> for Vec<u8> {
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType {
        self.as_slice().to_java(env)
    }
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
unsafe impl<'env> JavaConversionOwned<'env> for Vec<u8> {
    unsafe fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self {
        env.convert_byte_array(java).unwrap()
    }
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
        Ok(unsafe { Self::from_java(java.l()?.into_inner(), env) })
    }
}
