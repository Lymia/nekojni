use crate::{errors::*, utils::*, MacroCtx, SynTokenStream};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    parse2, spanned::Spanned, FnArg, GenericArgument, ImplItem, ImplItemMethod, ItemImpl, Pat,
    PatType, PathArguments, PathSegment, ReturnType, Signature, Type,
};

#[derive(Default)]
struct JavaClassComponents {
    sym_uid: usize,
    wrapper_funcs: SynTokenStream,
}
impl JavaClassComponents {
    fn gensym(&mut self, prefix: &str) -> Ident {
        let ident = ident!("{}_{}", prefix, self.sym_uid);
        self.sym_uid += 1;
        ident
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum FuncSelfMode {
    SelfRef,
    SelfMut,
    EnvRef,
    EnvMut,
    Static,
}

#[derive(Debug)]
enum FuncArgMode {
    ParamOwned(Type),
    ParamRef(Type),
    ParamMut(Type),
}
impl FuncArgMode {
    fn ty(&self) -> &Type {
        match self {
            FuncArgMode::ParamOwned(ty) => ty,
            FuncArgMode::ParamRef(ty) => ty,
            FuncArgMode::ParamMut(ty) => ty,
        }
    }
}

#[derive(Debug)]
enum FuncArg {
    SelfRef,
    SelfMut,
    EnvRef,
    EnvMut,
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
                    Ok(FuncArg::EnvMut)
                } else {
                    Ok(FuncArg::EnvRef)
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
                    Ok(FuncArg::SelfMut)
                } else {
                    Ok(FuncArg::SelfRef)
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
                FuncArg::SelfRef => self_mode = FuncSelfMode::SelfRef,
                FuncArg::SelfMut => self_mode = FuncSelfMode::SelfMut,
                FuncArg::EnvRef => self_mode = FuncSelfMode::EnvRef,
                FuncArg::EnvMut => self_mode = FuncSelfMode::EnvMut,
                FuncArg::ParamOwned(_) | FuncArg::ParamMut(_) => error(
                    param.span(),
                    "static methods must have &JNIEnv as the first arg",
                )?,
                FuncArg::ParamRef(_) => self_mode = FuncSelfMode::Static,
            }
        } else {
            match arg {
                FuncArg::SelfRef | FuncArg::SelfMut | FuncArg::EnvRef | FuncArg::EnvMut => {
                    error(param.span(), "self arg after first argument??")?
                }
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
    if !item.block.stmts.is_empty() {
        error(
            item.block.span(),
            "extern \"Java\" functions must have an empty body.",
        )?;
    }
    let (self_mode, args) = process_method_args(ctx, &mut item.sig)?;
    if self_mode == FuncSelfMode::EnvMut {
        error(
            item.sig.inputs.span(),
            "extern \"Java\" functions should not take self mutably.",
        )?;
    }

    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let (self_param, env, lt) = match self_mode {
        FuncSelfMode::EnvRef => (
            quote!(self: &#nekojni::JniRef<Self>),
            quote!(#nekojni::JniRef::env(self)),
            quote!(),
        ),
        FuncSelfMode::Static => (
            quote!(env: impl #std::convert::AsRef<#jni::JNIEnv<'envlt>>),
            quote!(env.as_ref()),
            quote!(<'envlt>),
        ),

        FuncSelfMode::SelfRef => error(
            item.sig.inputs.span(),
            "extern \"Java\" functions must take self as a `JniRef`.",
        )?,
        FuncSelfMode::SelfMut | FuncSelfMode::EnvMut => error(
            item.sig.inputs.span(),
            "extern \"Java\" functions should not take self mutably.",
        )?,
    };

    // Rewrite the function to be a proper proxy to Java code.
    let new_method = {
        let mut param_types: Vec<_> = args.iter().map(FuncArgMode::ty).collect();

        let mut param_names = Vec::new();
        let mut param_conversion = SynTokenStream::new();
        for arg in &args {
            let in_name = components.gensym("in");
            let java_name = components.gensym("java");
            let ty = arg.ty();

            let in_arg = match arg {
                FuncArgMode::ParamOwned(_) => quote!(&#in_name),
                FuncArgMode::ParamRef(_) => quote!(#in_name),
                FuncArgMode::ParamMut(_) => quote!(#in_name),
            };

            param_conversion.extend(quote! {
                let #java_name = <#ty as #nekojni::conversions::JavaConversion>::to_java(
                    #in_arg, &env,
                );
            });
            param_names.push(in_name);
        }

        let mut body = quote! {
            const PARAMS: &'static [#nekojni::signatures::Type<'static>] = &[
                #(<#param_types as #nekojni::conversions::JavaConversion>::JAVA_TYPE,)*
            ];

            let env = #env;
            #param_conversion
        };
        if self_mode == FuncSelfMode::Static {
            let wrapper_fn = components.gensym("wrapper_fn");
            body = quote! {
                fn #wrapper_fn(env: &#jni::JNIEnv, #(#param_names: #param_types,)*) {
                    #body
                }
                let env = #env;
                #wrapper_fn(env, #(#param_names,)*)
            };
        }
        parse2::<ImplItemMethod>(quote! {
            fn func #lt(#self_param, #(#param_names: #param_types,)*) {
                #body
            }
        })?
    };

    item.sig.abi = None;
    item.sig.generics = new_method.sig.generics;
    item.sig.inputs = new_method.sig.inputs;
    item.block = new_method.block;
    components.wrapper_funcs.extend(quote! {});

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
    if item.sig.generics.params.iter().next().is_some() {
        error(
            item.sig.generics.span(),
            "`#[jni_exports]` may not contain generic functions.",
        )?;
    }
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
