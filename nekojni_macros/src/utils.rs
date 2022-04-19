//! A module containing helper functions used throughout the macros implementation.

use crate::{
    errors::{Error, Result},
    MacroCtx,
};
use enumset::{EnumSet, EnumSetType};
use nekojni_utils::signatures::ClassName;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as SynTokenStream};
use quote::*;
use std::fmt::{Debug, Display};
use syn::{spanned::Spanned, *};

/// Creates an identifier with a format-like syntax.
macro_rules! ident {
    ($($tts:tt)*) => { syn::Ident::new(&format!($($tts)*), ::proc_macro2::Span::call_site()) }
}

/// Emits a `syn` based compile error.
pub fn error<T>(span: Span, message: impl Display) -> Result<T> {
    Err(Error::new(span, &message.to_string()))
}

/// Returns the actual type name of a path as a string.
pub fn last_path_segment(path: &Path) -> String {
    (&path.segments)
        .into_iter()
        .last()
        .expect("Empty path?")
        .ident
        .to_string()
}

/// Helpers for parsing interior attributes in the outer block.
const ATTR_OK_STR: &str = concat!(
    "(If you include this string in your crate, you are doing a bad, unstable thing.) ",
    "__",
    env!("CARGO_PKG_NAME"),
    "_attr_ok_d1c2d245c9024bfb893272fa6555e981_",
    env!("CARGO_PKG_VERSION"),
);

fn smart_err_attr(attr: SynTokenStream, item: SynTokenStream, error: &str) -> SynTokenStream {
    syn::Error::new(stream_span(if attr.is_empty() { item } else { attr }), error)
        .to_compile_error()
}
fn is_handler_valid(attr: SynTokenStream) -> bool {
    if attr.clone().into_iter().count() != 1 {
        return false;
    }
    parse2::<Lit>(attr)
        .ok()
        .map(|x| match x {
            Lit::Str(s) => s.value() == ATTR_OK_STR,
            _ => false,
        })
        .unwrap_or(false)
}
fn err_helper_attribute(
    error_str: &str,
    attr: SynTokenStream,
    item: SynTokenStream,
) -> SynTokenStream {
    if !is_handler_valid(attr.clone()) {
        smart_err_attr(attr, item, error_str)
    } else {
        SynTokenStream::new()
    }
}

/// Checks if an attribute has been processed via `mark_attribute_processed`.
///
/// Not public API, use [`derived_attr!`] instead.
#[doc(hidden)]
pub fn check_attr(error_str: &str, attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: SynTokenStream = item.into();
    let error = err_helper_attribute(error_str, attr.into(), item.clone());
    (quote! {
        #error
        #item
    })
    .into()
}

/// Creates a macro attribute that exists only to be processed by another macro attribute.
///
/// The macro will result in an error if it's used outside the macro. The macro must be
/// marked with [`mark_attribute_processed`] once processed to suppress this error.
macro_rules! derived_attr {
    (@error_str $attr:ident ($($head:tt)*) $inside:ident,) => {
        concat!(
            "#[", stringify!($attr), "] may only be used in a ",
            $($head)* "#[", stringify!($inside), "]",
            " block.",
        )
    };
    (@error_str $attr:ident ($($head:tt)*) $first:ident, $last:ident) => {
        concat!(
            "#[", stringify!($attr), "] may only be used in a ",
            $($head)* "#[", stringify!($first), "], or #[", stringify!($last), "]",
            " block.",
        )
    };
    (@error_str $attr:ident ($($head:tt)*) $inside:ident, $($rest:ident,)*) => {
        derived_attr!(@error_str $attr ("#[", stringify!($inside), "], ",) $($rest,)*)
    };
    ($event_name:ident, $($inside:ident),* $(,)?) => {
        #[proc_macro_attribute]
        pub fn $event_name(attr: TokenStream, item: TokenStream) -> TokenStream {
            const ERROR_STR: &str = derived_attr!(@error_str $event_name () $($inside,)*);
            crate::utils::check_attr(ERROR_STR, attr, item)
        }
    };
}

/// Marks an attribute as having been successfully processed.
///
/// See [`derived_attr!`].
pub fn mark_attribute_processed(attr: &mut Attribute) {
    attr.tokens = quote! { (#ATTR_OK_STR) }.into();
}

/// Creates a span for an entire TokenStream.
pub fn stream_span(attr: SynTokenStream) -> Span {
    let head_span = attr.clone().into_iter().next().unwrap().span();
    let tail_span = attr.into_iter().last().unwrap().span();
    head_span.join(tail_span).unwrap()
}

/// Dumps an [`EnumSet`] to a token stream.
pub(crate) fn enumset_to_toks<T: EnumSetType + Debug>(
    ctx: &MacroCtx,
    ty: SynTokenStream,
    set: EnumSet<T>,
) -> SynTokenStream {
    let nekojni_internal = &ctx.internal;

    let mut accum = quote!();
    for value in set {
        let ident = Ident::new(&format!("{value:?}"), Span::call_site());
        accum = quote!(#accum #ty::#ident |);
    }
    quote!(#nekojni_internal::enumset::enum_set!(#accum))
}

/// Parses a Java formatted class name.
pub fn parse_class_name<'a>(name: &str) -> Result<ClassName> {
    match ClassName::parse_java(name) {
        Ok(v) => Ok(v),
        Err(e) => error(Span::call_site(), format!("Could not parse class name: {e:?}")),
    }
}
