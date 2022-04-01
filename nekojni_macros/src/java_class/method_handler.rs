use crate::{errors::*, java_class::JavaClassCtx, utils::*, MacroCtx};
use proc_macro2::TokenStream as SynTokenStream;
use quote::{quote, quote_spanned};
use syn::{parse2, spanned::Spanned, FnArg, ImplItemMethod, Pat, ReturnType, Signature, Type};

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
    EnvRef(Type),
    EnvMut(Type),
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
                    Ok(FuncArg::EnvMut((*ref_tp.elem).clone()))
                } else {
                    Ok(FuncArg::EnvRef((*ref_tp.elem).clone()))
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
    components: &mut JavaClassCtx,
    sig: &mut Signature,
) -> Result<(FuncSelfMode, Vec<FuncArgMode>)> {
    if sig.inputs.is_empty() {
        error(
            sig.span(),
            "All Java-related functions must have at least one parameter.",
        )?;
    }

    let nekojni_internal = &ctx.internal;

    let mut is_first = true;
    let mut self_mode = FuncSelfMode::Static;
    let mut args = Vec::new();
    for param in &mut sig.inputs {
        let arg = FuncArg::from_param(param)?;
        if is_first {
            match arg {
                FuncArg::SelfRef => self_mode = FuncSelfMode::SelfRef,
                FuncArg::SelfMut => self_mode = FuncSelfMode::SelfMut,
                FuncArg::EnvRef(ty) => {
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#ty>();
                            #nekojni_internal::check_jniref(promise);
                        });
                    self_mode = FuncSelfMode::EnvRef;
                }
                FuncArg::EnvMut(ty) => {
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#ty>();
                            #nekojni_internal::check_jniref(promise);
                        });
                    self_mode = FuncSelfMode::EnvMut;
                }
                FuncArg::ParamOwned(ty) | FuncArg::ParamMut(ty) | FuncArg::ParamRef(ty) => {
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#ty>();
                            #nekojni_internal::check_jnienv(promise);
                        });
                    self_mode = FuncSelfMode::Static
                }
            }
        } else {
            match arg {
                FuncArg::SelfRef | FuncArg::SelfMut | FuncArg::EnvRef(_) | FuncArg::EnvMut(_) => {
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
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
) -> Result<bool> {
    if !item.block.stmts.is_empty() {
        error(
            item.block.span(),
            "extern \"Java\" functions must have an empty body.",
        )?;
    }
    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;
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

    // Setup important spans
    let item_span = item.span();
    let params_span = item.sig.span();
    let output_span = item.sig.output.span();

    // Setup the parameter types.
    let (self_param, env, lt) = match self_mode {
        FuncSelfMode::EnvRef => (
            quote_spanned!(params_span => self: &#nekojni::JniRef<Self>),
            quote_spanned!(params_span => #nekojni::JniRef::env(self)),
            quote_spanned!(params_span => /* nothing */),
        ),
        FuncSelfMode::Static => (
            quote_spanned!(params_span => env: impl #std::convert::AsRef<#nekojni::JniEnv<'env>>),
            quote_spanned!(params_span => env),
            quote_spanned!(params_span => <'env>),
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

    // Parse the type signature of the function.
    let mut param_types: Vec<_> = args.iter().map(FuncArgMode::ty).collect();

    let mut param_names = Vec::new();
    let mut param_conversion = SynTokenStream::new();
    for arg in &args {
        let in_name = components.gensym("in");
        let java_name = components.gensym("java");
        let ty = arg.ty();

        let in_arg = match arg {
            FuncArgMode::ParamOwned(_) => quote_spanned!(item_span => &#in_name),
            FuncArgMode::ParamRef(_) => quote_spanned!(item_span => #in_name),
            FuncArgMode::ParamMut(_) => quote_spanned!(item_span => #in_name),
        };

        param_conversion.extend(quote_spanned! { item_span =>
            let #java_name = <#ty as #nekojni::conversions::JavaConversion>::to_java(
                #in_arg, env,
            );
        });
        param_names.push(in_name);
    }

    let ret_ty = match &item.sig.output {
        ReturnType::Default => quote_spanned! { output_span => () },
        ReturnType::Type(_, ty) => quote_spanned! { output_span => #ty },
    };

    // Generate the body of the function
    let cache_field_name = components.gensym("cached_method_id");
    if self_mode == FuncSelfMode::Static {
        components.generated_exports.extend(quote_spanned! { item_span =>
            #cache_field_name: #nekojni_internal::OnceCache<#jni::objects::JStaticMethodID<'env>>,
        });
    } else {
        components
            .generated_exports
            .extend(quote_spanned! { item_span =>
                #cache_field_name: #nekojni_internal::OnceCache<#jni::objects::JMethodID<'env>>,
            });
    }

    let mut body = quote_spanned! { item_span =>
        const PARAMS: &'static [#nekojni::signatures::Type<'static>] = &[
            #(<#param_types as #nekojni::conversions::JavaConversion>::JAVA_TYPE,)*
        ];
        const RETURN_TY: #nekojni::signatures::ReturnType<'static> =
            <#ret_ty as #nekojni_internal::ImportReturnTy>::JAVA_TYPE;
        const SIGNATURE: #nekojni::signatures::MethodSig<'static> =
            #nekojni::signatures::MethodSig {
                ret_ty: RETURN_TY,
                params: #nekojni::signatures::StaticList::Borrowed(PARAMS),
            };

        let env = #env;
        #param_conversion

        static JNI_RET_CACHE: #nekojni_internal::OnceCache<#jni::signature::JavaType> =
            #nekojni_internal::OnceCache::new();
        static SIGNATURE_CACHE: #nekojni_internal::OnceCache<#jni::strings::JNIString> =
            #nekojni_internal::OnceCache::new();

        let jni_ret_cache = JNI_RET_CACHE.init(|| {
            SIGNATURE.ret_ty.into()
        });
        let signature_name = SIGNATURE_CACHE.init(|| {
            SIGNATURE.display_jni().to_string().into()
        });

        //let cache = #nekojni_internal::jni_ref::get_cache(&env);

        todo!()
    };
    if self_mode == FuncSelfMode::Static {
        let wrapper_fn = components.gensym("wrapper_fn");
        body = quote_spanned! { item_span =>
            fn #wrapper_fn(env: #nekojni::JniEnv, #(#param_names: #param_types,)*) {
                #body
            }
            let env = *env.as_ref();
            #wrapper_fn(env, #(#param_names,)*)
        };
    }

    // Generate the function in the additional impl block
    let new_method = parse2::<ImplItemMethod>(quote_spanned! { item_span =>
        fn func #lt(#self_param, #(#param_names: #param_types,)*) {
            #body
        }
    })?;
    item.sig.abi = None;
    item.sig.generics = new_method.sig.generics;
    item.sig.inputs = new_method.sig.inputs;
    item.block = new_method.block;
    components.generated_impls.extend(quote! { #item });

    Ok(true)
}

fn method_wrapper_exported(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
) -> Result<bool> {
    item.sig.abi = None;

    let args = process_method_args(ctx, components, &mut item.sig)?;

    Ok(false)
}

pub(crate) fn method_wrapper(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
) -> Result<bool> {
    if item.sig.generics.params.iter().next().is_some() {
        error(
            item.sig.generics.span(),
            "`#[jni_exports]` may not contain generic functions.",
        )?;
    }

    // process the method's attributes
    for attr in &mut item.attrs {
        if last_path_segment(&attr.path) == "jni" {
            mark_attribute_processed(attr);
        }
    }

    // process the method itself
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
