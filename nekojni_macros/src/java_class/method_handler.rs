use crate::{errors::*, java_class::JavaClassCtx, utils::*, MacroCtx};
use darling::FromAttributes;
use enumset::EnumSet;
use nekojni_utils::{
    signatures::{ClassName, MethodName},
    MFlags,
};
use proc_macro2::{Ident, Span, TokenStream as SynTokenStream, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse2, spanned::Spanned, Attribute, FnArg, ImplItemMethod, Lifetime, Pat, ReturnType,
    Signature, Type,
};

// TODO: Rewrite methods to allow better interop between `JniRef` and `JniRefMut`.

#[derive(Debug, FromAttributes)]
#[darling(attributes(jni))]
struct FunctionAttrs {
    #[darling(default)]
    rename: Option<String>,
    #[darling(default)]
    constructor: bool,

    #[darling(default, rename = "__njni_direct_export")]
    direct_export: Option<String>,
    #[darling(default, rename = "__njni_export_module_info")]
    export_module_info: Option<String>,

    #[darling(default, rename = "internal")]
    acc_internal: bool,
    #[darling(default, rename = "protected")]
    acc_protected: bool,
    #[darling(default, rename = "private")]
    acc_private: bool,
    #[darling(default, rename = "open")]
    acc_open: bool,
    #[darling(default, rename = "abstract")]
    acc_abstract: bool,
    #[darling(default, rename = "synchronized")]
    acc_synchronized: bool,
}
impl FunctionAttrs {
    fn check_internal_used(&self) -> Result<()> {
        if self.direct_export.is_some() || self.export_module_info.is_some() {
            error(Span::call_site(), "Attrs starting with `__njni_` are internal.")?;
        }
        Ok(())
    }

    fn parse_flags(
        &self,
        span: &impl Spanned,
        self_mode: &FuncSelfMode,
    ) -> Result<EnumSet<MFlags>> {
        if !self.acc_open && self.acc_abstract {
            error(span.span(), "`abstract` methods must also be `open`.")?;
        }
        if (self.acc_internal as u8) + (self.acc_protected as u8) + (self.acc_private as u8) > 1 {
            error(span.span(), "Only one of `internal`, `protected` or `private` may be used.")?;
        }

        let mut flags = EnumSet::new();
        if self.acc_protected {
            flags.insert(MFlags::Protected);
        } else if self.acc_private {
            flags.insert(MFlags::Private);
        } else if !self.acc_internal {
            flags.insert(MFlags::Public);
        }
        if !self.acc_open {
            flags.insert(MFlags::Final);
        }
        if self.acc_abstract {
            flags.insert(MFlags::Abstract);
        }
        if self.acc_synchronized {
            flags.insert(MFlags::Synchronized);
        }

        if let FuncSelfMode::Static = self_mode {
            flags.insert(MFlags::Static);
        }

        Ok(flags)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FuncSelfMode {
    SelfRef,
    SelfMut,
    EnvRef(Type),
    EnvMut(Type),
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

fn process_params_java(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
    args: &[FuncArgMode],
) -> Result<(Vec<Type>, Vec<Type>, Vec<Ident>, Vec<Ident>, SynTokenStream, Type, Type)> {
    let nekojni = &ctx.nekojni;

    // Setup important spans
    let item_span = item.span();

    // Process input arguments
    let param_tys: Vec<_> = args
        .iter()
        .map(FuncArgMode::ty)
        .map(|x| rewrite_self(x, &components.self_ty))
        .collect();
    let param_tys_elided: Vec<_> = param_tys.iter().map(elide_lifetimes).collect();

    let mut params = Vec::new();
    let mut params_java = Vec::new();
    let mut java_convert = SynTokenStream::new();
    for arg in args {
        let in_name = components.gensym("in");
        let java_name = components.gensym("java");
        let ty = rewrite_self(arg.ty(), &components.self_ty);

        let in_arg = match arg {
            FuncArgMode::ParamOwned(_) => quote_spanned!(item_span => &#in_name),
            FuncArgMode::ParamRef(_) => quote_spanned!(item_span => #in_name),
            FuncArgMode::ParamMut(_) => quote_spanned!(item_span => #in_name),
        };

        java_convert.extend(quote_spanned! { item_span =>
            let #java_name = <#ty as #nekojni::conversions::JavaConversion>::to_java_value(
                #in_arg, env,
            );
        });
        params.push(in_name);
        params_java.push(java_name);
    }

    let ret_ty = match &item.sig.output {
        ReturnType::Default => parse2::<Type>(quote! { () })?,
        ReturnType::Type(_, ty) => rewrite_self(&ty, &components.self_ty),
    };
    let ret_ty_elided = elide_lifetimes(&ret_ty);

    Ok((
        param_tys,
        param_tys_elided,
        params,
        params_java,
        java_convert,
        ret_ty,
        ret_ty_elided,
    ))
}

fn method_wrapper_java(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
    attrs: &FunctionAttrs,
) -> Result<bool> {
    if !item.block.stmts.is_empty() {
        error(item.block.span(), "extern \"Java\" functions must have an empty body.")?;
    }
    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;

    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let self_ty = components.self_ty.clone();

    // Java method name
    let java_name = match &attrs.rename {
        None => heck::AsLowerCamelCase(item.sig.ident.to_string()).to_string(),
        Some(name) => name.clone(),
    };

    // Setup important spans
    let item_span = item.span();
    let sig_span = item.sig.span();
    let output_span = item.sig.output.span();

    // Parse the type signature of the function.
    let (param_tys, param_tys_elided, params, params_java, java_convert, ret_ty, ret_ty_elided) =
        process_params_java(ctx, components, item, &args)?;
    let lt = check_only_lt(item)?.unwrap_or_else(|| Lifetime::new("'env", Span::call_site()));

    // Setup the parameter types.
    let (self_param, env) = match self_mode {
        FuncSelfMode::EnvRef(_) => (
            quote_spanned!(sig_span => self: &#nekojni::JniRef<#lt, #self_ty>),
            quote_spanned!(sig_span => #nekojni::JniRef::env(self)),
        ),
        FuncSelfMode::Static => (
            quote_spanned!(sig_span => env: impl #std::borrow::Borrow<#nekojni::JniEnv<#lt>>),
            quote_spanned!(sig_span => env),
        ),

        FuncSelfMode::SelfRef => error(
            item.sig.inputs.span(),
            "extern \"Java\" functions must take self as a `JniRef`.",
        )?,
        FuncSelfMode::SelfMut | FuncSelfMode::EnvMut(_) => error(
            item.sig.inputs.span(),
            "extern \"Java\" functions should not take self mutably.",
        )?,
    };

    // Generate the body of the function
    let rust_class_name = item.sig.ident.to_string();
    let (wrap_params, wrap_call, call_method) = match self_mode {
        FuncSelfMode::EnvRef(_) => (
            quote_spanned! { item_span => this: #jni::sys::jobject, },
            quote_spanned! { item_span => #nekojni::JniRef::this(self).into_inner(), },
            quote_spanned! { item_span =>
                let ret_val = env.call_method(
                    this, #java_name, SIGNATURE, &[#(#params_java,)*],
                );
            },
        ),
        FuncSelfMode::Static => {
            let class_name = &components.class_name;
            (
                quote_spanned! { item_span => },
                quote_spanned! { item_span => },
                quote_spanned! { item_span =>
                    let ret_val = env.call_static_method(
                        #class_name, #java_name, SIGNATURE, &[#(#params_java,)*],
                    );
                },
            )
        }
        _ => unreachable!(),
    };

    let wrapper_fn = components.gensym("wrapper_fn");
    let new_method = parse2::<ImplItemMethod>(quote_spanned! { item_span =>
        fn func<#lt>(
            #self_param,
            #(#params: impl #std::borrow::Borrow<#param_tys>,)*
        ) -> #ret_ty {
            fn #wrapper_fn<#lt>(
                env: #nekojni::JniEnv<#lt>,
                #wrap_params
                #(#params: &#param_tys,)*
            ) -> #ret_ty {
                const SIGNATURE: &'static str = #nekojni_internal::constcat_const!(
                    "(",
                    #(<#param_tys_elided as #nekojni::conversions::JavaConversionType>
                        ::JNI_TYPE,)*
                    ")",
                    <#ret_ty_elided as #nekojni_internal::ImportReturnTy>::JNI_TYPE,
                );

                #java_convert

                #call_method

                #nekojni_internal::ImportReturnTy::from_return_ty(
                    #rust_class_name, env, ret_val.map_err(|x| x.into()),
                )
            }

            let env = #env;
            #wrapper_fn(
                *#std::borrow::Borrow::borrow(&env),
                #wrap_call
                #(#std::borrow::Borrow::borrow(&#params),)*
            )
        }
    })?;

    // Generate the function in the additional impl block
    item.sig.abi = None;
    item.sig.generics = new_method.sig.generics;
    item.sig.inputs = new_method.sig.inputs;
    item.block = new_method.block;
    components.generated_impls.extend(quote! { #item });

    Ok(true)
}

fn constructor_wrapper_java(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
    attrs: &FunctionAttrs,
) -> Result<bool> {
    if !item.block.stmts.is_empty() {
        error(item.block.span(), "extern \"Java\" functions must have an empty body.")?;
    }
    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;

    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let self_ty = components.self_ty.clone();

    // Setup important spans
    let item_span = item.span();
    let sig_span = item.sig.span();
    let output_span = item.sig.output.span();

    // Check the function signature.
    match self_mode {
        FuncSelfMode::Static => (),
        _ => error(item.sig.inputs.span(), "extern \"Java\" constructors must be static.")?,
    }

    // Parse the type signature of the function.
    let (param_tys, param_tys_elided, params, params_java, java_convert, ret_ty, ret_ty_elided) =
        process_params_java(ctx, components, item, &args)?;

    // Generate the body of the function
    let java_class_name = components.class_name.clone();
    let rust_class_name = item.sig.ident.to_string();

    let wrapper_fn = components.gensym("wrapper_fn");
    let new_method = parse2::<ImplItemMethod>(quote_spanned! { item_span =>
        fn func<'env>(
            env: impl #std::borrow::Borrow<#nekojni::JniEnv<'env>>,
            #(#params: impl #std::borrow::Borrow<#param_tys>,)*
        ) -> #ret_ty {
            fn #wrapper_fn(
                env: #nekojni::JniEnv,
                #(#params: &#param_tys,)*
            ) -> #ret_ty {
                const SIGNATURE: &'static str = #nekojni_internal::constcat_const!(
                    "(",
                    #(<#param_tys_elided as #nekojni::conversions::JavaConversionType>::JNI_TYPE,)*
                    ")V",
                );

                #java_convert

                let ret_val = env.new_object(
                    #java_class_name, SIGNATURE, &[#(#params_java,)*],
                );

                #nekojni_internal::ImportCtorReturnTy::<#self_ty>::from_return_ty(
                    #rust_class_name, env, ret_val.map_err(|x| x.into()),
                )
            }

            #wrapper_fn(
                *#std::borrow::Borrow::borrow(&env),
                #(#std::borrow::Borrow::borrow(&#params),)*
            )
        }
    })?;

    // Generate the function in the additional impl block
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
    attrs: &FunctionAttrs,
) -> Result<bool> {
    item.sig.abi = None;

    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;

    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let self_ty = components.self_ty.clone();

    // Java method name
    let rust_name = &item.sig.ident;
    let rust_name_str = rust_name.to_string();
    let java_name = match &attrs.rename {
        None => heck::AsLowerCamelCase(item.sig.ident.to_string()).to_string(),
        Some(name) => name.clone(),
    };

    // Setup important spans
    let item_span = item.span();
    let sig_span = item.sig.span();
    let output_span = item.sig.output.span();

    // Parse function access
    let m_flags = attrs.parse_flags(&item.sig, &self_mode)?;
    if m_flags.contains(MFlags::Abstract) {
        error(sig_span, "Concrete methods cannot be `abstract`.")?;
    }
    if !m_flags.contains(MFlags::Final) {
        if let FuncSelfMode::SelfMut | FuncSelfMode::EnvMut(_) = &self_mode {
            error(sig_span, "`open` methods cannot take `self` mutably.")?;
        }
    }

    // Parse the type signature of the function.
    let param_tys: Vec<_> = args
        .iter()
        .map(FuncArgMode::ty)
        .map(|x| rewrite_self(x, &components.self_ty))
        .collect();
    let param_tys_elided: Vec<_> = param_tys.iter().map(elide_lifetimes).collect();

    let mut params_java = Vec::new();
    let mut params_param = Vec::new();
    for arg in &args {
        params_java.push(components.gensym("java"));
        params_param.push(components.gensym("param"));
    }

    let ret_ty = match &item.sig.output {
        ReturnType::Default => parse2::<Type>(quote! { () })?,
        ReturnType::Type(_, ty) => rewrite_self(&ty, &self_ty),
    };
    let ret_ty_elided = elide_lifetimes(&ret_ty);

    let lt = check_only_lt(item)?.unwrap_or_else(|| Lifetime::new("'env", Span::call_site()));

    // Copy the method for `open` functions into a private impl.
    let rust_name = if attrs.acc_open {
        let exported_method = components.gensym(&rust_name_str);
        let mut item_copy = item.clone();
        item_copy.sig.ident = exported_method.clone();
        components
            .generated_private_impls
            .extend(quote! { #item_copy });
        quote! { #exported_method }
    } else {
        quote! { #rust_name }
    };

    // Extract various important parameters
    let (extra_param, extra_param_ident, extra_param_java, extract_ref) = match &self_mode {
        FuncSelfMode::SelfRef => (
            quote! { id_param: u32, },
            quote! { id_param, },
            quote! { "I", },
            quote_spanned! { sig_span =>
                <#nekojni::JniRef<#lt, #self_ty> as #nekojni_internal::ExtractSelfParam<#lt>>
                    ::extract(env, this, id_param)
            },
        ),
        FuncSelfMode::SelfMut => (
            quote! { id_param: u32, },
            quote! { id_param, },
            quote! { "I", },
            quote_spanned! { sig_span =>
                <#nekojni::JniRefMut<#lt, #self_ty> as #nekojni_internal::ExtractSelfParam<#lt>>
                    ::extract(env, this, id_param)
            },
        ),
        FuncSelfMode::EnvRef(ty) | FuncSelfMode::EnvMut(ty) => {
            let ty = rewrite_self(&ty, &self_ty);
            (
                quote! { id_param: u32, },
                quote! { id_param, },
                quote! { "I", },
                quote_spanned! { sig_span =>
                    <#ty as #nekojni_internal::ExtractSelfParam<#lt>>
                        ::extract(env, this, id_param)
                },
            )
        }
        FuncSelfMode::Static => (quote!(), quote!(), quote!(), quote!()),
    };
    let (self_param, mut fn_call_body, is_static) = match &self_mode {
        FuncSelfMode::SelfRef | FuncSelfMode::EnvRef(_) => (
            quote_spanned! { sig_span => this: #jni::sys::jobject },
            quote_spanned! { sig_span =>
                let this_ref = unsafe { #extract_ref };
                this_ref.#rust_name(#(#params_param,)*)
            },
            false,
        ),
        FuncSelfMode::SelfMut | FuncSelfMode::EnvMut(_) => (
            quote_spanned! { sig_span => this: #jni::sys::jobject },
            quote_spanned! { sig_span =>
                let mut this_ref = unsafe { #extract_ref };
                this_ref.#rust_name(#(#params_param,)*)
            },
            false,
        ),
        FuncSelfMode::Static => (
            quote_spanned! { sig_span => this: #jni::sys::jclass },
            quote_spanned! { sig_span =>
                #self_ty::#rust_name(env, #(#params_param,)*)
            },
            true,
        ),
    };
    for (i, arg) in args.iter().enumerate() {
        let name_java = &params_java[i];
        let name_param = &params_param[i];
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
    let native_export = components.gensym("NATIVE_EXPORT");
    let wrapper_name = components.gensym(&rust_name_str);
    let entry_point_name = components.gensym(&format!("entry_point_{rust_name_str}"));
    let exported_method = components.gensym("EXPORTED_METHOD");

    let method_sig = components.gensym("METHOD_SIG");
    let method_sig_native = components.gensym("METHOD_SIG_NATIVE");

    let access = enumset_to_toks(&ctx, quote!(#nekojni_internal::MFlags), m_flags);

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
            const #method_sig: &'static str = #nekojni_internal::constcat_const!(
                "(",
                #java_sig_params
                ")",
                <#ret_ty_elided as #nekojni_internal::MethodReturn>::JNI_RETURN_TYPE
            );
            pub const #exported_method: #nekojni_internal::exports::ExportedItem =
                #nekojni_internal::exports::ExportedItem::NativeMethodWrapper {
                    flags: #access,
                    name: #java_name,
                    signature: #method_sig,
                    native_name: #rust_name_str,
                    native_signature: #method_sig_native,
                    has_id_param: !#is_static,
                };
            pub const #native_export: #nekojni_internal::exports::RustNativeMethod =
                #nekojni_internal::exports::RustNativeMethod {
                    name: #rust_name_str,
                    sig: #method_sig_native,
                    fn_ptr: #entry_point_name as *mut #std::ffi::c_void,
                    is_static: #is_static,
                };
        });
    components
        .native_methods
        .push(quote_spanned! { sig_span => __njni_priv::#native_export });
    components
        .exports
        .push(quote_spanned! { sig_span => __njni_priv::#exported_method });

    // Create the wrapper function for `open` functions
    if attrs.acc_open {
        item.block.stmts.clear();
        method_wrapper_java(ctx, components, item, attrs)?;
    }

    Ok(attrs.acc_open)
}

pub(crate) fn method_wrapper(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
) -> Result<bool> {
    // process the method's attributes
    let attrs: FunctionAttrs = FromAttributes::from_attributes(&item.attrs)?;
    for attr in &mut item.attrs {
        if last_path_segment(&attr.path) == "jni" {
            mark_attribute_processed(attr);
        }
    }
    if !components.is_internal {
        attrs.check_internal_used()?;
    }

    // process export to CLI tool
    if let Some(class_name) = &attrs.export_module_info {
        let class_name = match ClassName::parse_java(class_name) {
            Ok(v) => v,
            Err(e) => error(Span::call_site(), format!("Could not parse class name: {e:?}"))?,
        };
        let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or(format!("unknown"));
        let version = std::env::var("CARGO_PKG_VERSION")
            .unwrap_or(format!("0"))
            .replace(".", ";");
        let export_name = format!(
            "__njni_modinfo_v1__{}_{}",
            ClassName::new(vec![pkg_name], &version).display_jni_export(),
            class_name.display_jni_export(),
        );

        let temp_item = parse2::<ImplItemMethod>(quote! {
            #[no_mangle]
            #[export_name = #export_name]
            fn func() { }
        })?;
        item.attrs.extend(temp_item.attrs);

        return Ok(false);
    }

    // process the method itself
    if let Some(abi) = &item.sig.abi {
        if let Some(abi) = &abi.name {
            let abi = abi.value();
            if abi == "Java" {
                if attrs.constructor {
                    return constructor_wrapper_java(ctx, components, item, &attrs);
                } else {
                    return method_wrapper_java(ctx, components, item, &attrs);
                }
            }
        }
    }
    method_wrapper_exported(ctx, components, item, &attrs)
}
