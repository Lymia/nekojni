use std::sync::atomic::{AtomicUsize, Ordering};

static LABEL_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LabelID(usize);
impl LabelID {
    pub fn new() -> LabelID {
        LabelID(LABEL_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct MethodWriter {}

enum SimpleBytecode {}
