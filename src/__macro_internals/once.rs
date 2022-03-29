use parking_lot::Once;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct OnceCache<T> {
    once: Once,
    cache: AtomicPtr<T>,
}
impl<T> OnceCache<T> {
    pub const fn new() -> Self {
        OnceCache {
            once: Once::new(),
            cache: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    pub fn init(&self, func: fn() -> T) -> &T {
        self.once.call_once(|| {
            self.cache
                .store(Box::into_raw(Box::new(func())), Ordering::SeqCst);
        });
        unsafe { &*self.cache.load(Ordering::SeqCst) }
    }
}
impl<T> Drop for OnceCache<T> {
    fn drop(&mut self) {
        let ptr = self.cache.get_mut();
        if !ptr.is_null() {
            std::mem::drop(unsafe { Box::from_raw(ptr) })
        }
    }
}
unsafe impl<T> Send for OnceCache<T> where T: Send {}
unsafe impl<T> Sync for OnceCache<T> where T: Sync {}
impl<T> Default for OnceCache<T> {
    fn default() -> Self {
        OnceCache::new()
    }
}
