#![allow(deprecated)]

use crate::{errors::*, java_class::*};
use jni::objects::{JObject, JValue};
use parking_lot::{
    lock_api::{ArcRwLockReadGuard, ArcRwLockWriteGuard},
    RawRwLock,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

mod sealed {
    pub trait Sealed {}
}

// TODO: Deadlock detection of some sort.
// TODO: Sort out locks from the same thread/JNIEnv more cleanly.

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
    Read(ArcRwLockReadGuard<RawRwLock, T>),
    Write(ArcRwLockWriteGuard<RawRwLock, T>),
}

/// A pointer type holding a JNI environment and a an exported object.
///
/// It holds the same lifetime parameters as a [`JniEnv`]. These should normally be elided as this
/// should not often be used in contexts where there is much ambiguity.
pub struct JniRef<'env, T, R: JniRefType = JniRefRead> {
    this: JObject<'env>,
    inner: InnerRef<T>,
    env: JniEnv<'env>,
    phantom: PhantomData<R>,
}
impl<'env, T: JavaClass<'env>, R: JniRefType> JniRef<'env, T, R> {
    /// Returns the underlying [`JObject`] associated with this pointer.
    pub fn this(this: &Self) -> JObject<'env> {
        this.this
    }

    /// Returns the [`JniEnv`] associated with this pointer.
    pub fn env(&self) -> JniEnv<'env> {
        self.env
    }
}

impl<'env, T: JavaClass<'env>> JniRef<'env, T> {
    /// Upgrades this [`JniRef`] into a [`JniRefMut`]. As this requires an owning reference, this
    /// can only usually be used in practice with references returned from Java functions.
    pub fn upgrade_ref(self) -> JniRefMut<'env, T> {
        JniRef {
            this: self.this,
            inner: match self.inner {
                InnerRef::Default => {
                    panic!("internal error: ugprading `JniRef` with no Rust contents.")
                }
                InnerRef::Read(read) => {
                    todo!()
                }
                InnerRef::Write(write) => InnerRef::Write(write),
            },
            env: self.env,
            phantom: PhantomData,
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

impl<'a, 'env: 'a, T: JavaClass<'env>> AsRef<JniEnv<'env>> for &'a JniRef<'env, T> {
    fn as_ref(&self) -> &JniEnv<'env> {
        &self.env
    }
}

/// Creates a new [`JniRef`] from a JNI environment and a java object containing an ID.
pub fn new_rust<'env, T: RustContents<'env>>(
    env: JniEnv<'env>,
    this_class: &str, // TODO
    this: JObject<'env>,
    id: Option<u32>,
) -> Result<JniRef<'env, T>> {
    let id = match id {
        Some(id) => id,
        None => match env.get_field(this, T::ID_FIELD, "I")? {
            JValue::Int(i) => i as u32,
            _ => unreachable!(),
        },
    };
    let manager = env.get_id_manager::<T>();
    let lock = manager.get(id)?;
    let inner = InnerRef::Read(lock.read_arc_recursive());
    Ok(JniRef { this, inner, env, phantom: PhantomData })
}

/// Creates a new [`JniRef`] from a JNI environment and a java object.
pub fn new_wrapped<'env, T: JavaClass<'env>>(
    env: JniEnv<'env>,
    this: JObject<'env>,
) -> Result<JniRef<'env, T>> {
    Ok(JniRef { this, inner: InnerRef::Default, env, phantom: PhantomData })
}

/// A [`JniRef`] that allows read-write access to its contents.
pub type JniRefMut<'env, T> = JniRef<'env, T, JniRefWrite>;

#[inline(never)]
fn mut_ptr_fail() -> ! {
    panic!("internal error: read-only lock in `JniRefMut`?")
}
