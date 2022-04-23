use crate::{
    errors::*,
    java_class::{method_handler::FunctionAttrs, JavaClassCtx},
    utils::*,
};
use enumset::EnumSet;
use nekojni_utils::{
    signatures::{ClassName, MethodName},
    MFlags,
};
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use syn::{
    parse2, spanned::Spanned, Expr, FnArg, ImplItemMethod, Lifetime, Pat, ReturnType, Signature,
    Stmt, Type,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FuncSelfMode {
    SelfRef,
    SelfMut,
    EnvRef(Type),
    EnvMut(Type),
    Static,
}

#[derive(Debug)]
pub enum FuncArgMode {
    ParamOwned(Type),
    ParamRef(Type),
    ParamMut(Type),
}
impl FuncArgMode {
    pub fn ty(&self) -> &Type {
        match self {
            FuncArgMode::ParamOwned(ty) => ty,
            FuncArgMode::ParamRef(ty) => ty,
            FuncArgMode::ParamMut(ty) => ty,
        }
    }
}

#[derive(Debug)]
pub enum FuncArg {
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

    pub fn from_param(param: &FnArg) -> Result<FuncArg> {
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

pub fn check_method_empty(item: &ImplItemMethod) -> Result<()> {
    if item.block.stmts.is_empty() {
        Ok(())
    } else if item.block.stmts.len() == 1 {
        let stmt = &item.block.stmts[0];
        let is_valid = match stmt {
            Stmt::Expr(expr) => match expr {
                Expr::Macro(macro_call) => {
                    macro_call.attrs.is_empty()
                        && macro_call.mac.tokens.is_empty()
                        && (macro_call.mac.path.is_ident("unimplemented")
                            || macro_call.mac.path.is_ident("unreachable"))
                }
                _ => false,
            },
            _ => false,
        };
        if !is_valid {
            error(item.block.span(), "extern \"Java\" functions must have an empty body.")
        } else {
            Ok(())
        }
    } else {
        error(item.block.span(), "extern \"Java\" functions must have an empty body.")
    }
}

pub fn process_method_args(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    sig: &mut Signature,
) -> Result<(FuncSelfMode, Vec<FuncArgMode>)> {
    if sig.inputs.is_empty() {
        error(sig.span(), "All Java-related functions must have at least one parameter.")?;
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
                    let check_ty = elide_lifetimes(&ty);
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#check_ty>();
                            #nekojni_internal::check_jniref(promise);
                        });
                    self_mode = FuncSelfMode::EnvRef(ty);
                }
                FuncArg::EnvMut(ty) => {
                    let check_ty = elide_lifetimes(&ty);
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#check_ty>();
                            #nekojni_internal::check_jniref(promise);
                        });
                    self_mode = FuncSelfMode::EnvMut(ty);
                }
                FuncArg::ParamOwned(ty) => {
                    let check_ty = elide_lifetimes(&ty);
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#check_ty>();
                            #nekojni_internal::check_jnienv(promise);
                        });
                    self_mode = FuncSelfMode::Static
                }
                FuncArg::ParamMut(ty) | FuncArg::ParamRef(ty) => error(
                    param.span(),
                    "Unrecognized first parameter to an JNI function. The first parameter \
                    must be an owned reference to a `JniEnv`, a `JniRef` self parameter, or \
                    a normal self reference.",
                )?,
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

pub fn process_params_exported(
    _ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &ImplItemMethod,
    args: &[FuncArgMode],
) -> Result<(Vec<Type>, Vec<Type>, Vec<Ident>, Vec<Ident>, Type, Type)> {
    // Process input arguments
    let param_tys: Vec<_> = args
        .iter()
        .map(FuncArgMode::ty)
        .map(|x| rewrite_self(x, &components.self_ty))
        .collect();
    let param_tys_elided: Vec<_> = param_tys.iter().map(elide_lifetimes).collect();

    let mut params_java = Vec::new();
    let mut params_rust = Vec::new();
    for _ in args {
        params_java.push(components.gensym("java"));
        params_rust.push(components.gensym("param"));
    }

    let ret_ty = match &item.sig.output {
        ReturnType::Default => parse2::<Type>(quote! { () })?,
        ReturnType::Type(_, ty) => rewrite_self(&ty, &components.self_ty),
    };
    let ret_ty_elided = elide_lifetimes(&ret_ty);

    Ok((param_tys, param_tys_elided, params_java, params_rust, ret_ty, ret_ty_elided))
}

pub struct ExportedFunction {
    pub java_name: String,
}

pub fn export_function(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
    attrs: &FunctionAttrs,
    m_flags: EnumSet<MFlags>,
    export_to_java: bool,
    self_mode: &FuncSelfMode,
    args: &[FuncArgMode],
) -> Result<ExportedFunction> {
    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let self_ty = components.self_ty.clone();
    let sig_span = item.sig.span();

    // Java name
    let rust_name = match &attrs.override_rust_name {
        Some(x) => x,
        None => &item.sig.ident,
    };
    let rust_name_str = item.sig.ident.to_string();
    let java_name = match &attrs.rename {
        None => heck::AsLowerCamelCase(&rust_name_str).to_string(),
        Some(name) => name.clone(),
    };

    // Parse the type signature of the function.
    let (param_tys, param_tys_elided, params_java, params_rust, ret_ty, ret_ty_elided) =
        process_params_exported(ctx, components, item, args)?;
    let lt = check_only_lt(item)?.unwrap_or_else(|| Lifetime::new("'env", Span::call_site()));

    // Extract various important parameters
    let (extra_param, extra_param_ident, extra_param_java, extract_id_option) =
        if self_mode == &FuncSelfMode::Static || attrs.export_direct {
            (
                quote! {},
                quote! {},
                quote! {},
                quote! { let id_option = #std::option::Option::None; },
            )
        } else {
            (
                quote! { id_param: u32, },
                quote! { id_param, },
                quote! { "I", },
                quote! { let id_option = #std::option::Option::Some(id_param); },
            )
        };
    let extract_ref = match self_mode {
        FuncSelfMode::SelfRef => quote_spanned! { sig_span =>
            <#nekojni::JniRef<#lt, #self_ty> as #nekojni_internal::ExtractSelfParam<#lt>>
                ::extract(env, this, id_option)
        },
        FuncSelfMode::SelfMut => quote_spanned! { sig_span =>
            <#nekojni::JniRefMut<#lt, #self_ty> as #nekojni_internal::ExtractSelfParam<#lt>>
                ::extract(env, this, id_option)
        },
        FuncSelfMode::EnvRef(ty) | FuncSelfMode::EnvMut(ty) => {
            let ty = rewrite_self(&ty, &self_ty);
            quote_spanned! { sig_span =>
                <#ty as #nekojni_internal::ExtractSelfParam<#lt>>
                    ::extract(env, this, id_option)
            }
        }
        FuncSelfMode::Static => quote!(),
    };
    let (self_param, mut fn_call_body, is_static) = match self_mode {
        FuncSelfMode::SelfRef | FuncSelfMode::EnvRef(_) => (
            quote_spanned! { sig_span => this: #jni::sys::jobject },
            quote_spanned! { sig_span =>
                #extract_id_option
                let this_ref = unsafe { #extract_ref };
                this_ref.#rust_name(#(#params_rust,)*)
            },
            false,
        ),
        FuncSelfMode::SelfMut | FuncSelfMode::EnvMut(_) => (
            quote_spanned! { sig_span => this: #jni::sys::jobject },
            quote_spanned! { sig_span =>
                #extract_id_option
                let mut this_ref = unsafe { #extract_ref };
                this_ref.#rust_name(#(#params_rust,)*)
            },
            false,
        ),
        FuncSelfMode::Static => (
            quote_spanned! { sig_span => this: #jni::sys::jclass },
            quote_spanned! { sig_span =>
                #self_ty::#rust_name(env, #(#params_rust,)*)
            },
            true,
        ),
    };
    for (i, arg) in args.iter().enumerate() {
        let name_java = &params_java[i];
        let name_param = &params_rust[i];
        match arg {
            FuncArgMode::ParamOwned(ty) => {
                let ty = rewrite_self(arg.ty(), &components.self_ty);
                fn_call_body = quote_spanned! { sig_span =>
                    let #name_param =
                        <#ty as #nekojni::conversions::JavaConversionOwned<#lt>>
                            ::from_java(#name_java, env);
                    #fn_call_body
                };
            }
            FuncArgMode::ParamRef(ty) => {
                let ty = rewrite_self(arg.ty(), &components.self_ty);
                fn_call_body = quote_spanned! { sig_span =>
                    <#ty as #nekojni::conversions::JavaConversion<#lt>>::from_java_ref(
                        #name_java, env, |#name_param| {
                            #fn_call_body
                        }
                    )
                };
            }
            FuncArgMode::ParamMut(ty) => {
                let ty = rewrite_self(arg.ty(), &components.self_ty);
                fn_call_body = quote_spanned! { sig_span =>
                    <#ty as #nekojni::conversions::JavaConversion<#lt>>::from_java_mut(
                        #name_java, env, |#name_param| {
                            #fn_call_body
                        }
                    )
                };
            }
        }
    }

    // Prepare code fragments for generating the wrapper function
    let native_export = components.gensym_const("EXPORT_NATIVE_FN");
    let wrapper_name = components.gensym(&rust_name_str);
    let entry_point_name = components.gensym("native_fn_entry");
    let exported_method = components.gensym_const("EXPORT_JAVA_FN");

    let method_sig = components.gensym_const("METHOD_SIG");
    let method_sig_native = components.gensym_const("METHOD_SIG_NATIVE");

    let access = enumset_to_toks(&ctx, quote!(#nekojni_internal::MFlags), m_flags);
    let export_direct = export_to_java && (is_static || attrs.export_direct);

    // Handle internal feature to directly export the Java_*_initialize function.
    let (direct_export_attrs, early_init) = if let Some(class_name) = &attrs.direct_export {
        let class_name = match ClassName::parse_java(class_name) {
            Ok(v) => v,
            Err(e) => error(Span::call_site(), format!("Could not parse class name: {e:?}"))?,
        };
        let method_name = MethodName::new(class_name, &rust_name_str);
        let method_name = method_name.display_jni_export().to_string();

        (
            quote_spanned! { sig_span =>
                #[no_mangle]
                #[export_name = #method_name]
                pub // not technically an attr, but, this works anyway.
            },
            quote_spanned! { sig_span =>
                #nekojni_internal::early_init();
            },
        )
    } else {
        (quote!(), quote!())
    };

    // Generate the actual code.
    components
        .generated_private_items
        .extend(quote_spanned! { sig_span =>
            #[inline(never)] // helps make stack traces more understandable
            unsafe fn #wrapper_name<#lt>(
                env: #nekojni::JniEnv<#lt>,
                #self_param,
                #extra_param
                #(#params_java:
                    <#param_tys as #nekojni::conversions::JavaConversionType>::JavaType,)*
            ) -> #ret_ty {
                #fn_call_body
            }

            #direct_export_attrs
            extern "system" fn #entry_point_name<#lt>(
                env: #jni::JNIEnv<#lt>,
                #self_param,
                #extra_param
                #(#params_java:
                    <#param_tys as #nekojni::conversions::JavaConversionType>::JavaType,)*
            ) -> <#ret_ty as #nekojni_internal::MethodReturn>::ReturnTy {
                #early_init
                #nekojni_internal::__njni_entry_point::<#ret_ty, _>(
                    env,
                    |env| unsafe {
                        #wrapper_name(env, this, #extra_param_ident #(#params_java,)*)
                    },
                    crate::__njni_module_info::EXCEPTION_CLASS,
                )
            }
        });
    let java_sig_params = quote_spanned! { sig_span =>
        #(<#param_tys_elided as #nekojni::conversions::JavaConversionType>::JNI_TYPE,)*
    };
    let method_sig_body = quote_spanned! { sig_span =>
        const #method_sig: &'static str = #nekojni_internal::constcat_const!(
            "(",
            #java_sig_params
            ")",
            <#ret_ty_elided as #nekojni_internal::MethodReturn>::JNI_RETURN_TYPE
        );
    };
    let (method_sig, method_sig_native) = if is_static {
        (method_sig.clone(), method_sig.clone())
    } else {
        components
            .generated_private_items
            .extend(quote_spanned! { sig_span =>
                const #method_sig_native: &'static str = #nekojni_internal::constcat_const!(
                    "(",
                    #extra_param_java
                    #java_sig_params
                    ")",
                    <#ret_ty_elided as #nekojni_internal::MethodReturn>::JNI_RETURN_TYPE
                );
            });
        (method_sig, method_sig_native)
    };
    components
        .generated_private_items
        .extend(quote_spanned! { sig_span =>
            pub const #native_export: #nekojni_internal::exported_class::RustNativeMethod =
                #nekojni_internal::exported_class::RustNativeMethod {
                    name: #java_name,
                    sig: #method_sig_native,
                    fn_ptr: #entry_point_name as *mut #std::ffi::c_void,
                    is_static: #is_static,
                    export_direct_flags: #access,
                    export_direct: #export_direct,
                };
        });
    if export_to_java && !export_direct {
        components
            .generated_private_items
            .extend(quote_spanned! { sig_span =>
                #method_sig_body
                pub const #exported_method: #nekojni_internal::exported_class::ExportedItem =
                    #nekojni_internal::exported_class::ExportedItem::NativeMethodWrapper {
                        flags: #access,
                        name: #java_name,
                        signature: #method_sig,
                        native_name: #java_name,
                        native_signature: #method_sig_native,
                        has_id_param: !#is_static,
                    };
            });
        components
            .exports
            .push(quote_spanned! { sig_span => __njni_priv::#exported_method });
    } else if is_static {
        components.generated_private_items.extend(method_sig_body);
    }
    components
        .native_methods
        .push(quote_spanned! { sig_span => __njni_priv::#native_export });

    Ok(ExportedFunction { java_name })
}

pub fn check_func_init(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
    attrs: &FunctionAttrs,
    self_mode: &FuncSelfMode,
    exported: &ExportedFunction,
) -> Result<()> {
    if attrs.init {
        if item.sig.inputs.len() != 1 {
            error(
                item.sig.inputs.span(),
                "`#[jni(init)]` functions must have exactly one argument.",
            )?;
        }

        let nekojni_internal = &ctx.internal;
        let ret_ty = match &item.sig.output {
            ReturnType::Default => parse2::<Type>(quote! { () })?,
            ReturnType::Type(_, ty) => elide_lifetimes(&rewrite_self(&ty, &components.self_ty)),
        };
        components
            .generated_type_checks
            .extend(quote_spanned! { item.sig.output.span() =>
                let promise = #nekojni_internal::promise::<#ret_ty>();
                #nekojni_internal::check_return_is_void(promise);
            });

        match self_mode {
            FuncSelfMode::Static => components.static_init.push(exported.java_name.clone()),
            _ => components.instance_init.push(exported.java_name.clone()),
        }
    }
    Ok(())
}
