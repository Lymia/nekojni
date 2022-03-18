use jni::strings::JNIString;
use parking_lot::Once;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct JNIStrCache {
    once: Once,
    cache: AtomicPtr<JNIString>,
}
impl JNIStrCache {
    pub const fn new() -> Self {
        JNIStrCache {
            once: Once::new(),
            cache: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    pub fn init(&self, func: fn() -> JNIString) -> &'static JNIString {
        self.once.call_once(|| {
            self.cache
                .store(Box::into_raw(Box::new(func())), Ordering::SeqCst);
        });
        unsafe { &*self.cache.load(Ordering::SeqCst) }
    }
}
