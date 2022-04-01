use crate::errors::*;
use chashmap::CHashMap;
use jni::{strings::JNIString, JNIEnv, NativeMethod};
use lazy_static::lazy_static;
use parking_lot::{
    lock_api::{ArcRwLockUpgradableReadGuard, ArcRwLockWriteGuard},
    RawRwLock, RwLock,
};
use std::{ops::Deref, sync::Arc};

#[derive(Default)]
struct JniEnvData {}

#[derive(Default)]
struct JniEnvCache {
    is_initialized: bool,
    data: Arc<JniEnvData>,
}

lazy_static! {
    static ref CACHES: CHashMap<usize, Arc<RwLock<JniEnvCache>>> = CHashMap::new();
}

fn jni_new_ref(env: JNIEnv) -> Result<Arc<JniEnvData>> {
    let offset = env.get_native_interface() as usize;
    CACHES.alter(offset, |f| {
        if f.is_none() {
            Some(Arc::new(RwLock::new(JniEnvCache::default())))
        } else {
            f
        }
    });
    let cache_arc = CACHES
        .get(&offset)
        .expect("JNIEnv has already been shutdown????")
        .clone();
    let lock = cache_arc.upgradable_read_arc();
    if lock.is_initialized {
        Ok(lock.data.clone())
    } else {
        let mut write = ArcRwLockUpgradableReadGuard::upgrade(lock);
        write.is_initialized = true;

        let cache_offset = format!("{:x}", &CACHES as *const _ as usize);
        let (class_name, class_data) = nekojni_codegen::generate_shutdown_handler(&cache_offset);

        // register the native method handler
        env.register_native_methods(&class_name, &[NativeMethod {
            name: JNIString::from("native_shutdown"),
            sig: JNIString::from("()V"),
            fn_ptr: jni_shutdown_fn as *mut _,
        }])?;

        // define the shutdown hook class and install it
        let class_loader = env
            .call_static_method(
                "java/lang/ClassLoader",
                "getSystemClassLoader",
                "()Ljava/lang/ClassLoader;",
                &[],
            )?
            .l()?;
        env.define_class(&class_name, class_loader, &class_data)?;
        env.call_static_method(&class_name, "install", "()V", &[])?;

        // return a lock to the cache
        Ok(write.data.clone())
    }
}
fn jni_shutdown_fn(env: JniEnv) {
    crate::internal::panicking::catch_panic_jni(env, || jni_shutdown(env))
}
fn jni_shutdown(env: JniEnv) {
    CACHES
        .remove(&(env.get_native_interface() as usize))
        .expect("JNIEnv has already been shutdown?");
}

/// A wrapper for a [`JNIEnv`] that implements some additional functionality like method ID
/// caching.
#[derive(Copy, Clone)]
pub struct JniEnv<'env> {
    env: JNIEnv<'env>,
    cache: &'env JniEnvData,
}
impl<'env> JniEnv<'env> {
    /// Creates a new [`JniEnv`] wrapping this class.
    pub fn with_env<R>(env: JNIEnv, func: impl FnOnce(JniEnv) -> Result<R>) -> Result<R> {
        let data = jni_new_ref(env)?;
        func(JniEnv { env, cache: &data })
    }
}
impl<'env> Deref for JniEnv<'env> {
    type Target = JNIEnv<'env>;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}
