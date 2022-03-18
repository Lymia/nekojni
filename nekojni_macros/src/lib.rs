#[macro_use]
mod utils;
#[macro_use]
mod errors;
mod java_class;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as SynTokenStream};
use proc_macro_crate::{Error, FoundCrate};
use quote::quote;

struct MacroCtx {
    nekojni: SynTokenStream,
    internal: SynTokenStream,
    std: SynTokenStream,
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
        })
    }
}

#[proc_macro_attribute]
pub fn jni_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    try_syn!(java_class::jni_export(attr.into(), item.into())).into()
}
