use parking_lot::RwLock;

static DEFAULT_EXCEPTION_CLASS: RwLock<&'static str> = RwLock::new("java/lang/RuntimeException");

pub fn set_default_exception_class(class: &'static str) {
    *DEFAULT_EXCEPTION_CLASS.write() = class;
}
pub fn get_default_exception_class() -> &'static str {
    *DEFAULT_EXCEPTION_CLASS.read()
}