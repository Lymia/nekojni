use crate::MacroCtx;
use nekojni_signatures::ClassName;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn dump_class_name(ctx: &MacroCtx, class: &ClassName) -> TokenStream {
    let nekojni = &ctx.nekojni;
    let name = class.name;
    let package = class.package.as_slice();
    quote!(
        #nekojni::signatures::ClassName::new(&[#(#package,)*], #name)
    )
}
