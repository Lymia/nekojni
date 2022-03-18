use crate::{errors::*, utils::*, MacroCtx, SynTokenStream};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse2, spanned::Spanned, FnArg, GenericArgument, ImplItem, ImplItemMethod, ItemImpl, Pat,
    PatType, PathArguments, PathSegment, Signature, Type,
};

#[derive(Default)]
struct JavaClassComponents {
    wrapper_funcs: SynTokenStream,
}

#[derive(Copy, Clone, Debug)]
enum FuncSelfMode {
    InstanceRef,
    InstanceMut,
    Static,
}

#[derive(Debug)]
enum FuncArgMode {
    ParamOwned(Type),
    ParamRef(Type),
    ParamMut(Type),
}

#[derive(Debug)]
enum FuncArg {
    DirectSelfRef,
    DirectSelfMut,
    TypedSelfRef,
    TypedSelfMut,
    ParamOwned(Type),
    ParamRef(Type),
    ParamMut(Type),
}
impl FuncArg {
    fn check_any_type(tp: &Type) -> Result<FuncArg> {
        match tp {
            Type::Reference(ref_tp) => {
                if ref_tp.mutability.is_some() {
                    Ok(FuncArg::ParamMut((*ref_tp.elem).clone()))
                } else {
                    Ok(FuncArg::ParamRef((*ref_tp.elem).clone()))
                }
            }
            Type::Paren(paren) => Self::check_any_type(&paren.elem),
            Type::Group(group) => Self::check_any_type(&group.elem),
            Type::Macro(_) => error(tp.span(), "nekojni cannot currently parse macro types!"),
            _ => Ok(FuncArg::ParamOwned(tp.clone())),
        }
    }
    fn check_self_type(tp: &Type) -> Result<FuncArg> {
        match tp {
            Type::Reference(ref_tp) => {
                if ref_tp.mutability.is_some() {
                    Ok(FuncArg::TypedSelfMut)
                } else {
                    Ok(FuncArg::TypedSelfRef)
                }
            }
            Type::Paren(paren) => Self::check_any_type(&paren.elem),
            Type::Group(group) => Self::check_any_type(&group.elem),
            _ => error(tp.span(), "JNI functions may not take `self` by value."),
        }
    }

    fn from_param(param: &FnArg) -> Result<FuncArg> {
        match param {
            FnArg::Receiver(receiver) => {
                if receiver.reference.is_none() {
                    error(param.span(), "JNI functions may not take `self` by value.")
                } else if receiver.mutability.is_some() {
                    Ok(FuncArg::DirectSelfMut)
                } else {
                    Ok(FuncArg::DirectSelfRef)
                }
            }
            FnArg::Typed(ty) => match &*ty.pat {
                Pat::Ident(id) if id.ident.to_string() == "self" => Self::check_self_type(&ty.ty),
                _ => Self::check_any_type(&ty.ty),
            },
        }
    }
}

fn process_method_args(
    ctx: &MacroCtx,
    sig: &mut Signature,
) -> Result<(FuncSelfMode, Vec<FuncArgMode>)> {
    if sig.inputs.is_empty() {
        error(
            sig.span(),
            "All Java-related functions must have at least one parameter.",
        )?;
    }

    let nekojni = &ctx.nekojni;

    let mut is_first = true;
    let mut self_mode = FuncSelfMode::Static;
    let mut args = Vec::new();
    for param in &mut sig.inputs {
        let arg = FuncArg::from_param(param)?;
        if is_first {
            match arg {
                FuncArg::DirectSelfRef => {
                    *param = parse2(quote!(self: &#nekojni::java_class::JniRef<Self>))?;
                    self_mode = FuncSelfMode::InstanceRef;
                }
                FuncArg::DirectSelfMut => {
                    *param = parse2(quote!(self: &mut #nekojni::java_class::JniRef<Self>))?;
                    self_mode = FuncSelfMode::InstanceMut;
                }
                FuncArg::TypedSelfRef => self_mode = FuncSelfMode::InstanceRef,
                FuncArg::TypedSelfMut => self_mode = FuncSelfMode::InstanceMut,
                FuncArg::ParamOwned(_) | FuncArg::ParamMut(_) => error(
                    param.span(),
                    "static methods must have &JNIEnv as the first arg",
                )?,
                FuncArg::ParamRef(_) => self_mode = FuncSelfMode::Static,
            }
        } else {
            match arg {
                FuncArg::DirectSelfRef
                | FuncArg::DirectSelfMut
                | FuncArg::TypedSelfRef
                | FuncArg::TypedSelfMut => error(param.span(), "self arg after first argument??")?,
                FuncArg::ParamOwned(ty) => args.push(FuncArgMode::ParamOwned(ty)),
                FuncArg::ParamRef(ty) => args.push(FuncArgMode::ParamRef(ty)),
                FuncArg::ParamMut(ty) => args.push(FuncArgMode::ParamMut(ty)),
            }
        }
        is_first = false;
    }

    Ok((self_mode, args))
}

fn method_wrapper_java(
    ctx: &MacroCtx,
    components: &mut JavaClassComponents,
    item: &mut ImplItemMethod,
) -> Result<()> {
    item.sig.abi = None;

    if !item.block.stmts.is_empty() {
        error(
            item.block.span(),
            "extern \"Java\" functions must have an empty body.",
        )?;
    }
    let args = process_method_args(ctx, &mut item.sig)?;

    Ok(())
}

fn method_wrapper_exported(
    ctx: &MacroCtx,
    components: &mut JavaClassComponents,
    item: &mut ImplItemMethod,
) -> Result<()> {
    item.sig.abi = None;

    let args = process_method_args(ctx, &mut item.sig)?;

    Ok(())
}

fn method_wrapper(
    ctx: &MacroCtx,
    components: &mut JavaClassComponents,
    item: &mut ImplItemMethod,
) -> Result<()> {
    if let Some(abi) = &item.sig.abi {
        if let Some(abi) = &abi.name {
            let abi = abi.value();
            if abi == "Java" {
                return method_wrapper_java(ctx, components, item);
            }
        }
    }
    method_wrapper_exported(ctx, components, item)
}

pub fn jni_export(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let ctx = MacroCtx::new()?;
    let mut item = parse2::<ItemImpl>(item)?;
    let mut components = JavaClassComponents::default();

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
                if let Err(e) = method_wrapper(&ctx, &mut components, item) {
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
