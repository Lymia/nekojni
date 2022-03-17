use crate::*;
use jni::strings::JNIString;

impl<'a> From<ClassName<'a>> for JNIString {
    fn from(name: ClassName<'a>) -> Self {
        JNIString::from(name.display_jni().to_string())
    }
}
impl<'a> From<MethodSig<'a>> for JNIString {
    fn from(name: MethodSig<'a>) -> Self {
        JNIString::from(name.display_jni().to_string())
    }
}
impl<'a> From<Type<'a>> for JNIString {
    fn from(name: Type<'a>) -> Self {
        JNIString::from(name.display_jni().to_string())
    }
}
