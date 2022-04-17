#[macro_use]
mod utils;
#[macro_use]
mod errors;

mod java_class;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as SynTokenStream;
use proc_macro_crate::FoundCrate;
use quote::quote;
use std::sync::atomic::{AtomicUsize, Ordering};

fn chain_next() -> usize {
    static CHAIN_COUNT: AtomicUsize = AtomicUsize::new(1);
    CHAIN_COUNT.fetch_add(1, Ordering::SeqCst)
}

struct MacroCtx {
    nekojni: SynTokenStream,
    internal: SynTokenStream,
    std: SynTokenStream,
    jni: SynTokenStream,
}
impl MacroCtx {
    fn new() -> errors::Result<Self> {
        let crate_name = match proc_macro_crate::crate_name("nekojni") {
            Ok(FoundCrate::Name(v)) => ident!("{}", v),
            _ => ident!("nekojni"), // This is likely an example.
        };
        Ok(MacroCtx {
            nekojni: quote!( #crate_name ),
            internal: quote!( #crate_name::__macro_internals ),
            std: quote!( #crate_name::__macro_internals::std ),
            jni: quote!( #crate_name::__macro_internals::jni ),
        })
    }
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

derived_attr!(jni, jni_export, jni_import);
