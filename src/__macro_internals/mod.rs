mod object_id;

pub use crate::panicking::catch_panic_jni;
pub use object_id::IdManager;

pub struct ClassInfo {
    pub exception_class: &'static str,
}

pub use std;
