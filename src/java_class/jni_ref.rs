#![allow(deprecated)]

use crate::{__macro_internals::RustContents, errors::*, java_class::*};
use jni::objects::{JObject, JValue};
use parking_lot::{
    lock_api::{ArcRwLockUpgradableReadGuard, ArcRwLockWriteGuard},
    RawRwLock,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

mod sealed {
    pub trait Sealed {}
}

/// A marker trait used to represent a possible mode of a [`JniRef`].
pub trait JniRefType: sealed::Sealed {}

/// A marker trait for read-only [`JniRef`]s.
pub enum JniRefRead {}

/// A marker trait for read-write [`JniRef`]s
pub enum JniRefWrite {}

impl sealed::Sealed for JniRefRead {}
impl sealed::Sealed for JniRefWrite {}
impl JniRefType for JniRefRead {}
impl JniRefType for JniRefWrite {}

enum InnerRef<T> {
    Default,
    Read(ArcRwLockUpgradableReadGuard<RawRwLock, T>),
    Write(ArcRwLockWriteGuard<RawRwLock, T>),
}

/// A pointer type holding a JNI environment and a an exported object.
pub struct JniRef<'env, T: JavaClass<'env>, R: JniRefType = JniRefRead> {
    this: JObject<'env>,
    inner: InnerRef<T>,
    env: JNIEnv<'env>,
    phantom: PhantomData<R>,
    pub(crate) cache: T::Cache,
}
impl<'env, T: JavaClass<'env>, R: JniRefType> JniRef<'env, T, R> {
    /// Returns the underlying [`JObject`] associated with this pointer.
    pub fn this(this: &Self) -> JObject<'env> {
        this.this
    }

    /// Returns the [`JNIEnv`] associated with this pointer.
    pub fn env(this: &Self) -> JNIEnv<'env> {
        this.env
    }
}

impl<'env, T: JavaClass<'env>> JniRef<'env, T> {
    /// Upgrades this [`JniRef`] into a [`JniRefMut`]. As this requires an owning reference, this
    /// may only be used in practice with references returned from Java functions.
    pub fn upgrade(self) -> JniRefMut<'env, T> {
        JniRef {
            this: self.this,
            inner: match self.inner {
                InnerRef::Default => {
                    panic!("internal error: ugprading `JniRef` with no Rust contents.")
                }
                InnerRef::Read(read) => {
                    InnerRef::Write(ArcRwLockUpgradableReadGuard::upgrade(read))
                }
                InnerRef::Write(write) => InnerRef::Write(write),
            },
            env: self.env,
            phantom: PhantomData,
            cache: self.cache,
        }
    }
}

impl<'env, T: JavaClass<'env>, R: JniRefType> Deref for JniRef<'env, T, R> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            InnerRef::Default => T::default_ptr(),
            InnerRef::Read(p) => &p,
            InnerRef::Write(p) => &p,
        }
    }
}
impl<'env, T: JavaClass<'env>> DerefMut for JniRef<'env, T, JniRefWrite> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            InnerRef::Default => mut_ptr_fail(),
            InnerRef::Read(_) => mut_ptr_fail(),
            InnerRef::Write(p) => p.deref_mut(),
        }
    }
}

impl<'a, 'env: 'a, T: JavaClass<'env>> AsRef<JNIEnv<'env>> for &'a JniRef<'env, T> {
    fn as_ref(&self) -> &JNIEnv<'env> {
        &self.env
    }
}

/// Creates a new [`JniRef`] from a JNI environment and a java object containing an ID.
pub fn new_rust<'env, T: RustContents<'env>>(
    env: JNIEnv<'env>,
    this: JObject<'env>,
) -> Result<JniRef<'env, T>> {
    let id = match env.get_field(this, T::ID_FIELD, "I")? {
        JValue::Int(i) => i as u32,
        _ => unreachable!(),
    };
    let lock = T::get_manager().get(id)?;
    let inner = InnerRef::Read(lock.upgradable_read_arc());
    Ok(JniRef {
        this,
        inner,
        env,
        phantom: PhantomData,
        cache: T::Cache::default(),
    })
}

/// Creates a new [`JniRef`] from a JNI environment and a java object.
pub fn new_wrapped<'env, T: JavaClass<'env>>(
    env: JNIEnv<'env>,
    this: JObject<'env>,
) -> Result<JniRef<'env, T>> {
    Ok(JniRef {
        this,
        inner: InnerRef::Default,
        env,
        phantom: PhantomData,
        cache: T::Cache::default(),
    })
}

/// A [`JniRef`] that allows read-write access to its contents.
pub type JniRefMut<'env, T> = JniRef<'env, T, JniRefWrite>;

#[inline(never)]
fn mut_ptr_fail() -> ! {
    panic!("internal error: read-only lock in `JniRefMut`?")
}
