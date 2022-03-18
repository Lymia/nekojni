mod method_handler;

use crate::{errors::*, utils::*, MacroCtx};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse2, spanned::Spanned, ImplItem, ItemImpl};

#[derive(Default)]
pub(crate) struct JavaClassCtx {
    sym_uid: usize,
    wrapper_funcs: TokenStream,
}
impl JavaClassCtx {
    fn gensym(&mut self, prefix: &str) -> Ident {
        let ident = ident!("{}_{}", prefix, self.sym_uid);
        self.sym_uid += 1;
        ident
    }
}

pub fn jni_export(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let ctx = MacroCtx::new()?;
    let mut item = parse2::<ItemImpl>(item)?;
    let mut components = JavaClassCtx::default();

    if item.generics.params.iter().next().is_some() {
        error(
            item.generics.span(),
            "`#[jni_exports]` may not be used with generic impls.",
        )?;
    }

    let mut errors = Error::empty();
    for item in &mut item.items {
        match item {
            ImplItem::Method(item) => {
                if let Err(e) = method_handler::method_wrapper(&ctx, &mut components, item) {
                    errors = errors.combine(e);
                }
            }
            _ => {}
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(quote! {
        #item
    })
}
