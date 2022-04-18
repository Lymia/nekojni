use crate::errors::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, LitStr};

pub fn java_name_to_jni(toks: TokenStream) -> Result<TokenStream> {
    let string = parse2::<LitStr>(toks)?.value();
    let class_name = crate::utils::parse_class_name(&string)?;
    let class_name_str = class_name.display_jni().to_string();
    Ok(quote!(#class_name_str))
}
