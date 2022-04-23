use super::*;
use crate::objects::JArray;
use jni::objects::JObject;
use nekojni_utils::constcat_generic;

impl<'env, T: JavaConversionOwned<'env>> JavaConversionType for JArray<'env, T> {
    type JavaType = jobject;
    const JNI_TYPE: &'static str = constcat_generic!("[", T::JNI_TYPE);
}
unsafe impl<'env, T: JavaConversionOwned<'env>> JavaConversion<'env> for JArray<'env, T> {
    fn to_java(&self, _: JniEnv<'env>) -> Self::JavaType {
        self.obj().into_inner()
    }
    fn to_java_value(&self, _: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(self.obj())
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
unsafe impl<'env, T: JavaConversionOwned<'env>> JavaConversionOwned<'env> for JArray<'env, T> {
    unsafe fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self {
        JArray::from_obj(env, JObject::from(java))
    }
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
        Ok(unsafe { Self::from_java(java.l()?.into_inner(), env) })
    }
}
