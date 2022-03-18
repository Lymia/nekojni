use super::*;
use jni::objects::JObject;

impl JavaConversion for bool {
    const JAVA_TYPE: Type<'static> = Type::Boolean;
    type JavaType = jboolean;
    fn to_java(&self, _env: &JNIEnv) -> Self::JavaType {
        *self as u8
    }
    fn to_java_value(&self, env: &JNIEnv) -> JValue {
        JValue::Bool(self.to_java(env))
    }
    impl_borrowed_from_owned!();
    fn null() -> Self::JavaType {
        0
    }
}
impl JavaConversionOwned for bool {
    fn from_java(java: Self::JavaType, _: &JNIEnv) -> Self {
        java != 0
    }
    fn from_java_value(java: JValue, env: &JNIEnv) -> Result<Self> {
        if let JValue::Bool(value) = java {
            Ok(Self::from_java(value, env))
        } else {
            jni_bail!("Type error: expected Bool got {java:?}");
        }
    }
}

impl JavaConversion for str {
    const JAVA_TYPE: Type<'static> = Type::class(&["java", "lang"], "String");
    type JavaType = jstring;
    fn to_java(&self, env: &JNIEnv) -> Self::JavaType {
        env.new_string(self)
            .expect("could not convert jstring->String")
            .into_inner()
    }
    fn to_java_value(&self, env: &JNIEnv) -> JValue {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    fn from_java_ref<R>(java: Self::JavaType, env: &JNIEnv, func: impl FnOnce(&Self) -> R) -> R {
        let str = String::from_java(java, env);
        func(&str)
    }
    fn from_java_mut<R>(
        java: Self::JavaType,
        env: &JNIEnv,
        func: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let mut str = String::from_java(java, env);
        func(&mut str)
    }
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}

impl JavaConversion for String {
    const JAVA_TYPE: Type<'static> = Type::class(&["java", "lang"], "String");
    type JavaType = jstring;
    fn to_java(&self, env: &JNIEnv) -> Self::JavaType {
        str::to_java(self.as_str(), env)
    }
    fn to_java_value(&self, env: &JNIEnv) -> JValue {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    impl_borrowed_from_owned!();
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
impl JavaConversionOwned for String {
    fn from_java(java: Self::JavaType, env: &JNIEnv) -> Self {
        env.get_string(java.into())
            .expect("could not convert String->jstring")
            .into()
    }
    fn from_java_value(java: JValue, env: &JNIEnv) -> Result<Self> {
        if let JValue::Object(value) = java {
            Ok(Self::from_java(value.into_inner(), env))
        } else {
            jni_bail!("Type error: expected String got {java:?}");
        }
    }
}

impl JavaConversion for Vec<u8> {
    const JAVA_TYPE: Type<'static> = Type::Boolean.array();
    type JavaType = jbyteArray;
    fn to_java(&self, env: &JNIEnv) -> Self::JavaType {
        assert!(self.len() < jint::MAX as usize);
        let array = env
            .new_byte_array(self.len() as i32)
            .expect("could not create byte[]");
        env.set_byte_array_region(array, 0, bytemuck::cast_slice(self.as_slice()))
            .expect("Failed to copy data into byte array.");
        array
    }
    fn to_java_value(&self, env: &JNIEnv) -> JValue {
        JValue::Object(JObject::from(self.to_java(env)))
    }
    impl_borrowed_from_owned!();
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
impl JavaConversionOwned for Vec<u8> {
    fn from_java(java: Self::JavaType, env: &JNIEnv) -> Self {
        let mut new_vec = vec![0u8; env.get_array_length(java).unwrap() as usize];
        env.get_byte_array_region(java, 0, bytemuck::cast_slice_mut(new_vec.as_mut_slice()))
            .unwrap();
        new_vec
    }
    fn from_java_value(java: JValue, env: &JNIEnv) -> Result<Self> {
        if let JValue::Object(value) = java {
            Ok(Self::from_java(value.into_inner(), env))
        } else {
            jni_bail!("Type error: expected byte[] got {java:?}");
        }
    }
}

macro_rules! simple_conversion {
    ($(($rust_ty:ty, $jni_ty:ty, $java_ty:expr, $default:expr, $class:ident))*) => {$(
        impl JavaConversion for $rust_ty {
            const JAVA_TYPE: Type<'static> = $java_ty;
            type JavaType = $jni_ty;
            fn to_java(&self, _env: &JNIEnv) -> Self::JavaType {
                *self
            }
            fn to_java_value(&self, env: &JNIEnv) -> JValue {
                JValue::$class(self.to_java(env))
            }
            impl_borrowed_from_owned!();
            fn null() -> Self::JavaType {
                $default
            }
        }
        impl JavaConversionOwned for $rust_ty {
            fn from_java(java: Self::JavaType, _env: &JNIEnv) -> Self {
                java
            }
            fn from_java_value(java: JValue, env: &JNIEnv) -> Result<Self> {
                if let JValue::$class(value) = java {
                    Ok(Self::from_java(value, env))
                } else {
                    jni_bail!("Type error: expected {} got {java:?}", stringify!($class));
                }
            }
        }
    )*}
}
simple_conversion! {
    (f32, jfloat, Type::Float, 0.0, Float)
    (f64, jdouble, Type::Double, 0.0, Double)
}

macro_rules! numeric_conversion {
    ($(($rust_ty:ty, $jni_ty:ty, $java_ty:expr, $class:ident))*) => {$(
        impl JavaConversion for $rust_ty {
            const JAVA_TYPE: Type<'static> = $java_ty;
            type JavaType = $jni_ty;
            fn to_java(&self, _env: &JNIEnv) -> Self::JavaType {
                let val = *self;
                assert!(
                    <$rust_ty>::MAX != 0 || val <= <$jni_ty>::MAX as $rust_ty,
                    concat!(stringify!($rust_ty), " too large to convert to ", stringify!($jni_ty))
                );
                val as $jni_ty
            }
            fn to_java_value(&self, env: &JNIEnv) -> JValue {
                JValue::$class(self.to_java(env))
            }
            impl_borrowed_from_owned!();
            fn null() -> Self::JavaType {
                0
            }
        }
        impl JavaConversionOwned for $rust_ty {
            fn from_java(java: Self::JavaType, _env: &JNIEnv) -> Self {
                assert!(
                    <$rust_ty>::MAX != 0 || java < 0,
                    concat!(stringify!($rust_ty), " cannot be negative")
                );
                java as $rust_ty
            }
            fn from_java_value(java: JValue, env: &JNIEnv) -> Result<Self> {
                if let JValue::$class(value) = java {
                    Ok(Self::from_java(value, env))
                } else {
                    jni_bail!("Type error: expected {} got {java:?}", stringify!($class));
                }
            }
        }
    )*}
}
numeric_conversion! {
    (i8, jbyte, Type::Byte, Byte)
    (u8, jbyte, Type::Byte, Byte)
    (i16, jshort, Type::Short, Short)
    (u16, jshort, Type::Short, Short)
    (i32, jint, Type::Int, Int)
    (u32, jint, Type::Int, Int)
    (i64, jlong, Type::Long, Long)
    (u64, jlong, Type::Long, Long)
}
