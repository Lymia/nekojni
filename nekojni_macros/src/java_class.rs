use crate::{errors::*, utils::*, MacroCtx, SynTokenStream};
use proc_macro2::TokenStream;
use syn::{
    parse2, spanned::Spanned, FnArg, GenericArgument, ImplItemMethod, ItemImpl, Pat, PathArguments,
    PathSegment, Type,
};

#[derive(Default)]
struct JavaClassComponents {
    wrapper_funcs: SynTokenStream,
}

#[derive(Debug)]
enum HandlerArg {
    DirectSelfRef,
    DirectSelfMut,
    TypedSelfRef,
    TypedSelfMut,
    ParamOwned(Type),
    ParamRef(Type),
    ParamMut(Type),
}
impl HandlerArg {
    fn check_any_type(tp: &Type) -> Result<HandlerArg> {
        match tp {
            Type::Reference(ref_tp) => {
                if ref_tp.mutability.is_some() {
                    Ok(HandlerArg::ParamMut((*ref_tp.elem).clone()))
                } else {
                    Ok(HandlerArg::ParamRef((*ref_tp.elem).clone()))
                }
            }
            Type::Paren(paren) => Self::check_any_type(&paren.elem),
            Type::Group(group) => Self::check_any_type(&group.elem),
            Type::Macro(_) => error(tp.span(), "nekojni cannot currently parse macro types!"),
            _ => Ok(HandlerArg::ParamOwned(tp.clone())),
        }
    }
    fn check_self_type(tp: &Type) -> Result<HandlerArg> {
        match tp {
            Type::Reference(ref_tp) => {
                if ref_tp.mutability.is_some() {
                    Ok(HandlerArg::TypedSelfMut)
                } else {
                    Ok(HandlerArg::TypedSelfRef)
                }
            }
            Type::Paren(paren) => Self::check_any_type(&paren.elem),
            Type::Group(group) => Self::check_any_type(&group.elem),
            _ => error(tp.span(), "JNI functions may not take `self` by value."),
        }
    }

    fn from_param(param: &FnArg) -> Result<HandlerArg> {
        match param {
            FnArg::Receiver(receiver) => {
                if receiver.reference.is_none() {
                    error(param.span(), "JNI functions may not take `self` by value.")
                } else if receiver.mutability.is_some() {
                    Ok(HandlerArg::DirectSelfMut)
                } else {
                    Ok(HandlerArg::DirectSelfRef)
                }
            }
            FnArg::Typed(ty) => match &*ty.pat {
                Pat::Ident(id) if id.ident.to_string() == "self" => Self::check_self_type(&ty.ty),
                _ => Self::check_any_type(&ty.ty),
            },
        }
    }
}

fn method_wrapper(
    ctx: &MacroCtx,
    components: &mut JavaClassComponents,
    func: &mut ImplItemMethod,
) -> Result<()> {
    println!("{:#?}", func);
    todo!()
}

pub fn jni_export(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<ItemImpl>(item)?;

    if item.generics.params.iter().next().is_some() {
        error(
            item.generics.span(),
            "`#[jni_exports]` may not be used with generic impls.",
        )?;
    }

    println!("{:#?}", item);

    Ok(TokenStream::new())
}
