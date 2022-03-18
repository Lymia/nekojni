mod exports;
pub(crate) mod jni_ref;

pub use exports::*;
pub use jni_ref::{JniRef, JniRefMut};

use jni::JNIEnv;

/// A trait representing a Java class.
#[allow(deprecated)]
pub trait JavaClass: crate::__macro_internals::JavaClassImpl {}
