use jni::{sys::*, JNIEnv};

/// Main trait that converts between Java and Rust types.
pub trait JavaConversion {
    /// The fully qualified name of the Java type.
    const JAVA_NAME: &'static str;

    /// The type used for the exported function signature.
    type JavaType;

    /// Convert the Rust type into a Java type.
    fn into_java(self, env: &JNIEnv) -> Self::JavaType;

    /// Convert the Java type into a Rust type.
    fn from_java(java: Self::JavaType, env: &JNIEnv) -> Self;

    /// Returns the closest thing to a null value in this type. Used as a return type for an JNI
    /// function after returning an exception.
    fn null() -> Self::JavaType;
}

impl JavaConversion for bool {
    const JAVA_NAME: &'static str = "boolean";
    type JavaType = jboolean;
    fn into_java(self, _env: &JNIEnv) -> Self::JavaType {
        self as u8
    }
    fn from_java(java: Self::JavaType, _env: &JNIEnv) -> Self {
        java != 0
    }
    fn null() -> Self::JavaType {
        0
    }
}
impl JavaConversion for String {
    const JAVA_NAME: &'static str = "String";
    type JavaType = jstring;
    fn into_java(self, env: &JNIEnv) -> Self::JavaType {
        env.new_string(self)
            .expect("could not convert jstring->String")
            .into_inner()
    }
    fn from_java(java: Self::JavaType, env: &JNIEnv) -> Self {
        env.get_string(java.into())
            .expect("could not convert String->jstring")
            .into()
    }
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}
impl JavaConversion for Vec<u8> {
    const JAVA_NAME: &'static str = "byte[]";
    type JavaType = jbyteArray;
    fn into_java(self, env: &JNIEnv) -> Self::JavaType {
        assert!(self.len() < jint::MAX as usize);
        let array = env
            .new_byte_array(self.len() as i32)
            .expect("could not create byte[]");
        env.set_byte_array_region(array, 0, bytemuck::cast_slice(self.as_slice()))
            .expect("Failed to copy data into byte array.");
        array
    }
    fn from_java(java: Self::JavaType, env: &JNIEnv) -> Self {
        let mut new_vec = vec![0u8; env.get_array_length(java).unwrap() as usize];
        env.get_byte_array_region(java, 0, bytemuck::cast_slice_mut(new_vec.as_mut_slice()))
            .unwrap();
        new_vec
    }
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}

macro_rules! simple_conversion {
    ($(($rust_ty:ty, $jni_ty:ty, $java_ty:literal, $default:expr))*) => {$(
        impl JavaConversion for $rust_ty {
            const JAVA_NAME: &'static str = $java_ty;
            type JavaType = $jni_ty;
            fn into_java(self, _env: &JNIEnv) -> Self::JavaType {
                self
            }
            fn from_java(java: Self::JavaType, _env: &JNIEnv) -> Self {
                java
            }
            fn null() -> Self::JavaType {
                $default
            }
        }
    )*}
}
simple_conversion! {
    (f32, jfloat, "float", 0.0)
    (f64, jdouble, "double", 0.0)
}

macro_rules! numeric_conversion {
    ($(($rust_ty:ty, $jni_ty:ty, $java_ty:literal))*) => {$(
        impl JavaConversion for $rust_ty {
            const JAVA_NAME: &'static str = $java_ty;
            type JavaType = $jni_ty;
            fn into_java(self, _env: &JNIEnv) -> Self::JavaType {
                assert!(
                    <$rust_ty>::MAX != 0 || self <= <$jni_ty>::MAX as $rust_ty,
                    concat!(stringify!($rust_ty), " too large to convert to ", $java_ty)
                );
                self as $jni_ty
            }
            fn from_java(java: Self::JavaType, _env: &JNIEnv) -> Self {
                assert!(
                    <$rust_ty>::MAX != 0 || java < 0,
                    concat!(stringify!($rust_ty), " cannot be negative")
                );
                java as $rust_ty
            }
            fn null() -> Self::JavaType {
                0
            }
        }
    )*}
}
numeric_conversion! {
    (i8, jbyte, "byte")
    (u8, jbyte, "byte")
    (i16, jshort, "short")
    (u16, jshort, "short")
    (i32, jint, "int")
    (u32, jint, "int")
    (i64, jlong, "long")
    (u64, jlong, "long")
}
