use super::*;

impl JavaConversionType for bool {
    type JavaType = jboolean;
    const JNI_TYPE: &'static str = "Z";
}
unsafe impl<'env> JavaConversion<'env> for bool {
    fn to_java(&self, _env: JniEnv<'env>) -> Self::JavaType {
        *self as u8
    }
    fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
        JValue::Bool(self.to_java(env))
    }
    impl_borrowed_from_owned!('env);
    fn null() -> Self::JavaType {
        0
    }
}
unsafe impl<'env> JavaConversionOwned<'env> for bool {
    unsafe fn from_java(java: Self::JavaType, _: JniEnv<'env>) -> Self {
        java != 0
    }
    fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
        Ok(unsafe { Self::from_java(java.z()? as jboolean, env) })
    }
}

macro_rules! simple_conversion {
    ($(($rust_ty:ty, $jni_ty:ty, $jni_sig:expr, $default:expr, $class:ident, $conv:ident))*) => {$(
        impl JavaConversionType for $rust_ty {
            type JavaType = $jni_ty;
            const JNI_TYPE: &'static str = $jni_sig;
        }
        unsafe impl<'env> JavaConversion<'env> for $rust_ty {
            fn to_java(&self, _env: JniEnv<'env>) -> Self::JavaType {
                *self
            }
            fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
                JValue::$class(self.to_java(env))
            }
            impl_borrowed_from_owned!('env);
            fn null() -> Self::JavaType {
                $default
            }
        }
        unsafe impl<'env> JavaConversionOwned<'env> for $rust_ty {
            unsafe fn from_java(java: Self::JavaType, _env: JniEnv<'env>) -> Self {
                java
            }
            fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
                Ok(unsafe { Self::from_java(java.$conv()?, env) })
            }
        }
    )*}
}
simple_conversion! {
    (f32, jfloat, "F", 0.0, Float, f)
    (f64, jdouble, "D", 0.0, Double, d)
}

macro_rules! numeric_conversion {
    ($(($rust_ty:ty, $jni_ty:ty, $jni_sig:expr, $class:ident, $conv:ident))*) => {$(
        impl JavaConversionType for $rust_ty {
            type JavaType = $jni_ty;
            const JNI_TYPE: &'static str = $jni_sig;
        }
        unsafe impl<'env> JavaConversion<'env> for $rust_ty {
            fn to_java(&self, _env: JniEnv<'env>) -> Self::JavaType {
                let val = *self;
                assert!(
                    <$rust_ty>::MAX != 0 || val <= <$jni_ty>::MAX as $rust_ty,
                    concat!(stringify!($rust_ty), " too large to convert to ", stringify!($jni_ty))
                );
                val as $jni_ty
            }
            fn to_java_value(&self, env: JniEnv<'env>) -> JValue<'env> {
                JValue::$class(self.to_java(env))
            }
            impl_borrowed_from_owned!('env);
            fn null() -> Self::JavaType {
                0
            }
        }
        unsafe impl<'env> JavaConversionOwned<'env> for $rust_ty {
            unsafe fn from_java(java: Self::JavaType, _env: JniEnv<'env>) -> Self {
                assert!(
                    <$rust_ty>::MAX != 0 || java < 0,
                    concat!(stringify!($rust_ty), " cannot be negative")
                );
                java as $rust_ty
            }
            fn from_java_value(java: JValue<'env>, env: JniEnv<'env>) -> Result<Self> {
                Ok(unsafe { Self::from_java(java.$conv()?, env) })
            }
        }
    )*}
}
numeric_conversion! {
    (i8, jbyte, "B", Byte, b)
    (u8, jbyte, "B", Byte, b)
    (i16, jshort, "S", Short, s)
    (u16, jshort, "S", Short, s)
    (i32, jint, "I", Int, i)
    (u32, jint, "I", Int, i)
    (i64, jlong, "L", Long, j)
    (u64, jlong, "L", Long, j)
}
