use super::*;
use jni::objects::{JObject, JString};

impl<'env> JavaConversion<'env> for str {
    const JAVA_TYPE: Type<'static> = Type::class(&["java", "lang"], "String");
    type JavaType = JString<'env>;
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType {
        env.new_string(self)
            .expect("could not convert jstring->String")
    }
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    fn from_java_ref<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&Self) -> R,
    ) -> R {
        let str = String::from_java(java, env);
        func(&str)
    }
    fn from_java_mut<R>(
        java: Self::JavaType,
        env: JniEnv<'env>,
        func: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let mut str = String::from_java(java, env);
        func(&mut str)
    }
    fn null() -> Self::JavaType {
        String::null()
    }
}

impl<'env> JavaConversion<'env> for String {
    const JAVA_TYPE: Type<'static> = Type::class(&["java", "lang"], "String");
    type JavaType = JString<'env>;
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType {
        str::to_java(self.as_str(), env)
    }
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        JString::from(std::ptr::null_mut())
    }
}
impl<'env> JavaConversionOwned<'env> for String {
    fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self {
        env.get_string(java.into())
            .expect("could not convert String->jstring")
            .into()
    }
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
        Ok(Self::from_java(JString::from(java.l()?.into_inner()), env))
    }
}

impl<'env> JavaConversion<'env> for Vec<u8> {
    const JAVA_TYPE: Type<'static> = Type::Boolean.array();
    type JavaType = JObject<'env>;
    fn to_java(&self, env: JniEnv<'env>) -> Self::JavaType {
        assert!(self.len() < jint::MAX as usize);
        let array = env
            .new_byte_array(self.len() as i32)
            .expect("could not create byte[]");
        env.set_byte_array_region(array, 0, bytemuck::cast_slice(self.as_slice()))
            .expect("Failed to copy data into byte array.");
        JObject::from(array)
    }
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        JObject::from(std::ptr::null_mut())
    }
}
impl<'env> JavaConversionOwned<'env> for Vec<u8> {
    fn from_java(java: Self::JavaType, env: JniEnv<'env>) -> Self {
        let mut new_vec = vec![0u8; env.get_array_length(java.into_inner()).unwrap() as usize];
        env.get_byte_array_region(
            java.into_inner(),
            0,
            bytemuck::cast_slice_mut(new_vec.as_mut_slice()),
        )
        .unwrap();
        new_vec
    }
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
        Ok(Self::from_java(java.l()?, env))
    }
}
