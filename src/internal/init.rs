use crate::internal::jni_entry;

pub fn early_init() {
    match std::panic::catch_unwind(|| {}) {
        Ok(_) => {}
        Err(e) => jni_entry::panic_abort(e),
    }
}
