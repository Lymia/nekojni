#[macro_use]
mod utils;
#[macro_use]
mod errors;
mod jni_exports;

use proc_macro::TokenStream;

#[proc_macro_derive(JavaClass)]
pub fn derive_java_class(item: TokenStream) -> TokenStream {
    println!("item: \"{}\"", item.to_string());
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn jni_exports(attr: TokenStream, item: TokenStream) -> TokenStream {
    try_syn!(jni_exports::jni_exports(attr.into(), item.into())).into()
}
