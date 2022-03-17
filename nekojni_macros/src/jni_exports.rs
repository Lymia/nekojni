use crate::errors::*;
use proc_macro2::TokenStream;
use syn::{parse2, spanned::Spanned, ItemImpl};

pub fn jni_exports(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<ItemImpl>(item)?;

    if item.generics.params.iter().next().is_some() {
        return Err(Error::new(
            item.generics.span(),
            "#[jni_exports] may not be used with generic impls.",
        ));
    }

    Ok(TokenStream::new())
}
