#![allow(deprecated)]

use crate::{__macro_internals::IdManager, errors::*, java_class::*};
use jni::objects::JValue;
use parking_lot::{
    lock_api::{ArcRwLockReadGuard, ArcRwLockWriteGuard},
    RawRwLock, RwLock,
};
use std::ops::{Deref, DerefMut};

pub trait RustContents: Sized + Send + Sync + 'static {
    const ID_FIELD: &'static str;
    fn get_manager() -> &'static IdManager<RwLock<Self>>;
}

enum InnerRef<T> {
    None,
    Immutable(ArcRwLockReadGuard<RawRwLock, T>),
    Mutable(ArcRwLockWriteGuard<RawRwLock, T>),
}

/// A pointer type holding a JNI environment and a an exported object.
pub struct JniRef<'a, T: JavaClass> {
    inner: InnerRef<T>,
    env: &'a JNIEnv<'a>,
    jvm: T::JvmInterface,
}
impl<'a, T: JavaClass> JniRef<'a, T> {
    /// Creates a new [`JniRef`] from a JNI environment and a java object containing an ID.
    pub fn new(env: &'a JNIEnv<'a>, this: jobject, is_mut: bool) -> Result<Self>
    where T: RustContents {
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
        Ok(JniRef { inner, env, jvm })
    }

    /// Creates a new [`JniRef`] from a JNI environment and a java object.
    pub fn new_wrapped(env: &'a JNIEnv<'a>, this: jobject) -> Result<Self> {
        Ok(JniRef {
            inner: InnerRef::None,
            env,
            jvm: T::create_interface(env, this)?,
        })
    }

    /// Returns the [`JNIEnv`] associated with this pointer.
    pub fn env(this: &Self) -> &'a JNIEnv {
        this.env
    }

    /// Returns the JvmInterface associated with this pointer.
    pub fn interface(this: &Self) -> &T::JvmInterface {
        &this.jvm
    }
}

impl<'a, 'b: 'a, T: JavaClass> Deref for JniRef<'a, T>
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
impl<'a, 'b: 'a, T: JavaClass> DerefMut for JniRef<'a, T>
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

#[inline(never)]
fn imm_fail() -> ! {
    panic!("attempted mutable dereference of immutable pointer??")
}

#[inline(never)]
fn none_fail() -> ! {
    panic!("attempted dereference of JniRef with no Rust contents?")
}
