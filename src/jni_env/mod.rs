use crate::{errors::*, java_class::object_id::IdManager, JavaClass};
use chashmap::CHashMap;
use jni::{strings::JNIString, JNIEnv, NativeMethod};
use lazy_static::lazy_static;
use parking_lot::{lock_api::ArcRwLockUpgradableReadGuard, RwLock};
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::Deref,
    sync::Arc,
};

#[derive(Default)]
struct TransientCache<'env> {
    _phantom: PhantomData<&'env ()>,
}

#[derive(Default)]
struct JniEnvCacheData {
    rust_objects: CHashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

#[derive(Default)]
struct JniEnvCache {
    is_initialized: bool,
    data: Arc<JniEnvCacheData>,
}

lazy_static! {
    static ref CACHES: CHashMap<usize, Arc<RwLock<JniEnvCache>>> = CHashMap::new();
}

fn vm_offset(env: JNIEnv) -> Result<usize> {
    Ok(env.get_java_vm()?.get_java_vm_pointer() as usize)
}
fn jni_new_ref(env: JNIEnv) -> Result<Arc<JniEnvCacheData>> {
    let offset = vm_offset(env)?;

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
    let offset = vm_offset(*env).expect("Could not find offset?");
    CACHES
        .remove(&offset)
        .expect("JNIEnv has already been shutdown?");
}

/// A wrapper for a [`JNIEnv`] that implements some additional functionality used by nekojni.
#[derive(Copy, Clone)]
pub struct JniEnv<'env> {
    env: JNIEnv<'env>,
    cache: &'env JniEnvCacheData,
    transient_cache: &'env TransientCache<'env>,
}
impl<'env> JniEnv<'env> {
    /// Creates a new [`JniEnv`] wrapping this class.
    pub fn with_env<R>(env: JNIEnv, func: impl FnOnce(JniEnv) -> Result<R>) -> Result<R> {
        let data = jni_new_ref(env)?;
        let transient_cache = TransientCache::default();
        func(JniEnv { env, cache: &data, transient_cache: &transient_cache })
    }

    /// Returns an instance of an object for this entire JVM.
    ///
    /// The equivalent of static methods/variables should generally be stored in here if at all
    /// possible. This allows the instances to be per-VM rather than global to any JVM loaded in
    /// the same process.
    pub fn get_jvm_instance<T: Any + Send + Sync>(&self, create: impl FnOnce() -> T) -> Arc<T> {
        let id = TypeId::of::<T>();
        if !self.cache.rust_objects.contains_key(&id) {
            self.cache.rust_objects.alter(id, |f| {
                if f.is_some() {
                    f
                } else {
                    Some(Box::new(Arc::new(create())))
                }
            })
        }
        unsafe {
            let res = self
                .cache
                .rust_objects
                .get(&id)
                .unwrap()
                .downcast_ref_unchecked::<Arc<T>>()
                .clone();
            res
        }
    }

    /// Returns the ID manager for this type.
    pub(crate) fn get_id_manager<T: Any + Send + Sync>(&self) -> Arc<IdManager<RwLock<T>>> {
        self.get_jvm_instance(IdManager::new)
    }
}
impl<'env> Deref for JniEnv<'env> {
    type Target = JNIEnv<'env>;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}
