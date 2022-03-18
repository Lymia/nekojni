#![allow(deprecated)]

use crate::{__macro_internals::RustContents, errors::*, java_class::*};
use jni::objects::JValue;
use parking_lot::{
    lock_api::{ArcRwLockReadGuard, ArcRwLockWriteGuard},
    RawRwLock,
};
use std::ops::{Deref, DerefMut};

enum InnerRef<T> {
    None,
    Immutable(ArcRwLockReadGuard<RawRwLock, T>),
    Mutable(ArcRwLockWriteGuard<RawRwLock, T>),
}

/// A pointer type holding a JNI environment and a an exported object.
pub struct JniRef<T: JavaClass> {
    this: jobject,
    inner: InnerRef<T>,
    env: JNIEnv<'static>,
    jvm: T::JvmInterface,
}
impl<T: JavaClass> JniRef<T> {
    /// Returns the raw [`jobject`] associated with this pointer.
    pub(crate) fn this(this: &Self) -> jobject {
        this.this
    }

    /// Returns the [`JNIEnv`] associated with this pointer.
    pub fn env(this: &Self) -> &JNIEnv {
        &this.env
    }

    /// Returns the JvmInterface associated with this pointer.
    pub fn interface(this: &Self) -> &T::JvmInterface {
        &this.jvm
    }
}

impl<T: JavaClass> Deref for JniRef<T>
where T: RustContents
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            InnerRef::Immutable(p) => &p,
            InnerRef::Mutable(p) => &p,
            InnerRef::None => none_fail(),
        }
    }
}
impl<T: JavaClass> DerefMut for JniRef<T>
where T: RustContents
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            InnerRef::Immutable(_) => imm_fail(),
            InnerRef::Mutable(p) => p.deref_mut(),
            InnerRef::None => none_fail(),
        }
    }
}

impl<'a, T: JavaClass> AsRef<JNIEnv<'a>> for &'a JniRef<T> {
    fn as_ref(&self) -> &JNIEnv<'a> {
        JniRef::env(self)
    }
}

/// Creates a new [`JniRef`] from a JNI environment and a java object containing an ID.
pub unsafe fn new<T: RustContents>(env: &JNIEnv, this: jobject, is_mut: bool) -> Result<JniRef<T>> {
    let id = match env.get_field(this, T::ID_FIELD, "I")? {
        JValue::Int(i) => i as u32,
        _ => unreachable!(),
    };
    let jvm = T::create_interface(env, this)?;
    let lock = T::get_manager().get(id)?;
    let inner = if is_mut {
        InnerRef::Mutable(lock.write_arc())
    } else {
        InnerRef::Immutable(lock.read_arc())
    };
    Ok(JniRef {
        this,
        inner,
        env: { JNIEnv::from_raw(env.get_native_interface())? },
        jvm,
    })
}

/// Creates a new [`JniRef`] from a JNI environment and a java object.
pub unsafe fn new_wrapped<T: JavaClass>(env: &JNIEnv, this: jobject) -> Result<JniRef<T>> {
    Ok(JniRef {
        this,
        inner: InnerRef::None,
        env: { JNIEnv::from_raw(env.get_native_interface())? },
        jvm: T::create_interface(env, this)?,
    })
}

#[inline(never)]
fn imm_fail() -> ! {
    panic!("attempted mutable dereference of immutable pointer??")
}

#[inline(never)]
fn none_fail() -> ! {
    panic!("attempted dereference of JniRef with no Rust contents?")
}
