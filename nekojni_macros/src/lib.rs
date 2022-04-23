#![deny(unused_must_use, unused_imports)]

#[macro_use]
mod utils;
#[macro_use]
mod errors;

mod helper_macros;
mod java_class;

use proc_macro::TokenStream;
use std::sync::atomic::{AtomicUsize, Ordering};

fn chain_next() -> usize {
    static CHAIN_COUNT: AtomicUsize = AtomicUsize::new(1);
    CHAIN_COUNT.fetch_add(1, Ordering::SeqCst)
}

#[proc_macro_attribute]
pub fn jni_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    try_syn!(java_class::jni_export(attr.into(), item.into())).into()
}

#[proc_macro_attribute]
pub fn jni_export_internal(attr: TokenStream, item: TokenStream) -> TokenStream {
    try_syn!(java_class::jni_export_internal(attr.into(), item.into())).into()
}

#[proc_macro_attribute]
pub fn jni_import(attr: TokenStream, item: TokenStream) -> TokenStream {
    try_syn!(java_class::jni_import(attr.into(), item.into())).into()
}

#[proc_macro]
pub fn java_name_to_jni(item: TokenStream) -> TokenStream {
    try_syn!(helper_macros::java_name_to_jni(item.into())).into()
}

derived_attr!(jni, jni_export, jni_import);
