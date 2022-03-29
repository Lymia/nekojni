use jni::JNIEnv;
use std::ops::Deref;

/// A wrapper for a [`JNIEnv`] that implements some additional functionality like method ID
/// caching.
///
/// Unlike [`JNIEnv`], this does not directly contain a pointer, and hence is not [`Copy`]. It
/// should be used through an immutable borrow.
#[derive(Clone)]
pub struct JniEnv<'env> {
    env: JNIEnv<'env>,
}

impl<'env> Deref for JniEnv<'env> {
    type Target = JNIEnv<'env>;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}
