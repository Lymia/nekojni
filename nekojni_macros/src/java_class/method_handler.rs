use crate::{
    errors::*,
    java_class::{utils::*, JavaClassCtx},
    utils::*,
};
use darling::FromAttributes;
use enumset::EnumSet;
use nekojni_utils::{signatures::ClassName, MFlags};
use proc_macro2::{Ident, Span, TokenStream as SynTokenStream};
use quote::{quote, quote_spanned};
use syn::{parse2, spanned::Spanned, Abi, ImplItemMethod, Lifetime, ReturnType, Type};

// TODO: Rewrite methods to allow better interop between `JniRef` and `JniRefMut`.

#[derive(Clone, Debug, FromAttributes, Default)]
#[darling(attributes(jni))]
pub struct FunctionAttrs {
    #[darling(default)]
    pub rename: Option<String>,
    #[darling(default)]
    pub constructor: bool,
    #[darling(default)]
    pub init: bool,
    #[darling(default)]
    pub export_direct: bool,

    #[darling(default, rename = "__njni_direct_export")]
    pub direct_export: Option<String>,
    #[darling(default, rename = "__njni_export_module_info")]
    pub export_module_info: Option<String>,

    #[darling(default, rename = "public")]
    pub acc_public: bool,
    #[darling(default, rename = "internal")]
    pub acc_internal: bool,
    #[darling(default, rename = "protected")]
    pub acc_protected: bool,
    #[darling(default, rename = "private")]
    pub acc_private: bool,
    #[darling(default, rename = "open")]
    pub acc_open: bool,
    #[darling(default, rename = "abstract")]
    pub acc_abstract: bool,
    #[darling(default, rename = "synchronized")]
    pub acc_synchronized: bool,

    #[darling(skip)]
    pub acc_synthetic: bool,
    #[darling(skip)]
    pub override_rust_name: Option<Ident>,
}
impl FunctionAttrs {
    fn check_internal_used(&self) -> Result<()> {
        if self.direct_export.is_some() || self.export_module_info.is_some() {
            error(Span::call_site(), "Attrs starting with `__njni_` are internal.")?;
        }
        Ok(())
    }

    pub fn parse_flags(
        &self,
        span: &impl Spanned,
        self_mode: &FuncSelfMode,
        default_access: MFlags,
    ) -> Result<EnumSet<MFlags>> {
        if !self.acc_open && self.acc_abstract {
            error(span.span(), "`abstract` methods must also be `open`.")?;
        }
        if (self.acc_internal as u8)
            + (self.acc_protected as u8)
            + (self.acc_private as u8)
            + (self.acc_public as u8)
            > 1
        {
            error(span.span(), "Only one of `internal`, `protected` or `private` may be used.")?;
        }

        let mut flags = EnumSet::new();
        if self.acc_public {
            flags.insert(MFlags::Public);
        } else if self.acc_protected {
            flags.insert(MFlags::Protected);
        } else if self.acc_private {
            flags.insert(MFlags::Private);
        } else if !self.acc_internal {
            flags.insert(default_access);
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
        if self.acc_synthetic {
            flags.insert(MFlags::Synthetic);
        }

        if let FuncSelfMode::Static = self_mode {
            flags.insert(MFlags::Static);
        }

        Ok(flags)
    }
}

fn process_params_java(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &ImplItemMethod,
    args: &[FuncArgMode],
) -> Result<(Vec<Type>, Vec<Type>, Vec<Ident>, Vec<Ident>, SynTokenStream, Type, Type)> {
    let nekojni = &ctx.nekojni;
    let sig_span = item.span();

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
            FuncArgMode::ParamOwned(_) => quote_spanned!(sig_span => &#in_name),
            FuncArgMode::ParamRef(_) => quote_spanned!(sig_span => #in_name),
            FuncArgMode::ParamMut(_) => quote_spanned!(sig_span => #in_name),
        };

        java_convert.extend(quote_spanned! { sig_span =>
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
    check_method_empty(item)?;
    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;

    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let self_ty = components.self_ty.clone();
    let sig_span = item.sig.span();

    // Java method name
    let java_name = match &attrs.rename {
        None => heck::AsLowerCamelCase(item.sig.ident.to_string()).to_string(),
        Some(name) => name.clone(),
    };

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
            quote_spanned! { sig_span => this: #jni::sys::jobject, },
            quote_spanned! { sig_span => #nekojni::JniRef::this(self).into_inner(), },
            quote_spanned! { sig_span =>
                let ret_val = env.call_method(
                    this, #java_name, SIGNATURE, &[#(#params_java,)*],
                );
            },
        ),
        FuncSelfMode::Static => {
            let class_name = &components.class_name;
            (
                quote_spanned! { sig_span => },
                quote_spanned! { sig_span => },
                quote_spanned! { sig_span =>
                    let ret_val = env.call_static_method(
                        #class_name, #java_name, SIGNATURE, &[#(#params_java,)*],
                    );
                },
            )
        }
        _ => unreachable!(),
    };

    let wrapper_fn = components.gensym("wrapper_fn");
    let new_method = parse2::<ImplItemMethod>(quote_spanned! { sig_span =>
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
    check_method_empty(item)?;
    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;

    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let self_ty = components.self_ty.clone();
    let sig_span = item.sig.span();

    // Check the function signature.
    match self_mode {
        FuncSelfMode::Static => (),
        _ => error(item.sig.inputs.span(), "Constructors must be static.")?,
    }

    // Parse the type signature of the function.
    let (param_tys, param_tys_elided, params, params_java, java_convert, ret_ty, ret_ty_elided) =
        process_params_java(ctx, components, item, &args)?;
    let lt = check_only_lt(item)?.unwrap_or_else(|| Lifetime::new("'env", Span::call_site()));

    // Generate the body of the function
    let java_class_name = components.class_name.clone();
    let rust_class_name = item.sig.ident.to_string();

    let wrapper_fn = components.gensym("wrapper_fn");
    let new_method = parse2::<ImplItemMethod>(quote_spanned! { sig_span =>
        fn func<'env>(
            env: impl #std::borrow::Borrow<#nekojni::JniEnv<'env>>,
            #(#params: impl #std::borrow::Borrow<#param_tys>,)*
        ) -> #ret_ty {
            fn #wrapper_fn<#lt>(
                env: #nekojni::JniEnv<#lt>,
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

                #nekojni_internal::ImportCtorReturnTy::<#lt, #self_ty>::from_return_ty(
                    #rust_class_name,
                    env,
                    ret_val.map_err(|x| x.into()).map(|x| #jni::objects::JValue::Object(x)),
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
    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;

    let rust_name = &item.sig.ident;
    let rust_name_str = rust_name.to_string();
    let sig_span = item.sig.span();

    // Parse function access
    let m_flags = attrs.parse_flags(
        &item.sig,
        &self_mode,
        if attrs.init { MFlags::Private } else { MFlags::Public },
    )?;
    if m_flags.contains(MFlags::Abstract) {
        error(sig_span, "Concrete methods cannot be `abstract`.")?;
    }
    if !m_flags.contains(MFlags::Final) {
        if let FuncSelfMode::SelfMut | FuncSelfMode::EnvMut(_) = &self_mode {
            error(sig_span, "`open` methods cannot take `self` mutably.")?;
        }
    }

    // Copy the method for `open` functions into a private impl.
    let rust_name = if attrs.acc_open {
        let exported_method = components.gensym(&rust_name_str);
        let mut item_copy = item.clone();
        item_copy.sig.ident = exported_method.clone();
        components
            .generated_private_impls
            .extend(quote! { #item_copy });
        exported_method
    } else {
        rust_name.clone()
    };

    // Export the function to Java
    let mut attrs_copy = attrs.clone();
    attrs_copy.override_rust_name = Some(rust_name);
    let exported =
        export_function(ctx, components, item, &attrs_copy, m_flags, true, &self_mode, &args)?;

    // Check if this is an init function
    check_func_init(ctx, components, item, attrs, &self_mode, &exported)?;

    // Create the wrapper function for `open` functions
    if attrs.acc_open {
        item.block.stmts.clear();
        method_wrapper_java(ctx, components, item, attrs)?;
    }

    Ok(attrs.acc_open)
}

fn constructor_wrapper_exported(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
    attrs: &FunctionAttrs,
) -> Result<bool> {
    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;

    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let rust_name = &item.sig.ident;
    let rust_name_str = rust_name.to_string();
    let self_ty = components.self_ty.clone();
    let sig_span = item.sig.span();

    // Check the function signature.
    match self_mode {
        FuncSelfMode::Static => (),
        _ => error(item.sig.inputs.span(), "Constructors must be static.")?,
    }

    // Parse function access
    let mut m_flags = attrs.parse_flags(&item.sig, &self_mode, MFlags::Public)?;
    if m_flags.contains(MFlags::Abstract) {
        error(sig_span, "Constructors methods cannot be `abstract`.")?;
    }
    m_flags.remove(MFlags::Final);

    // Copy the constructor function into a private impl.
    let rust_name = {
        let exported_method = components.gensym(&rust_name_str);
        let mut item_copy = item.clone();
        item_copy.sig.ident = exported_method.clone();
        components
            .generated_private_impls
            .extend(quote! { #item_copy });
        quote! { #exported_method }
    };

    // Parse the type signature of the function.
    let (param_tys, param_tys_elided, params_java, params_rust, ret_ty, ret_ty_elided) =
        process_params_exported(ctx, components, item, &args)?;
    let lt = check_only_lt(item)?.unwrap_or_else(|| Lifetime::new("'env", Span::call_site()));

    // Create the wrapper function that actually creates the object
    let wrapper_rust_name = components.gensym("native_ctor");
    let wrapper_java_name = components.gensym_java("native_ctor");
    let synthetic_ty = components.gensym("SyntheticTy");
    let params_class_name = components.gensym_class("ConstructorParamsStore");

    let mut ctor_wrapper = parse2::<ImplItemMethod>(quote_spanned! { sig_span =>
        #[jni(private, rename = #wrapper_java_name, export_direct)]
        fn #wrapper_rust_name<#lt>(
            env: #nekojni::JniEnv<#lt>,
            #(#params_rust: #param_tys,)*
        ) -> #nekojni::Result<
            <#ret_ty as #nekojni_internal::ConstructorReturnTy<#lt, #synthetic_ty>>::ReturnType,
        > {
            let result = Self::#rust_name(env, #(#params_rust,)*);
            <#ret_ty as #nekojni_internal::ConstructorReturnTy<#lt, #synthetic_ty>>
                ::ctor_new(result, #params_class_name, env)
        }
    })?;
    assert!(!method_wrapper(ctx, components, &mut ctor_wrapper, true)?);
    components
        .generated_private_items
        .extend(quote_spanned! { sig_span =>
            enum #synthetic_ty { }
            impl #nekojni_internal::SyntheticTyType for #synthetic_ty {
                const CLASS_NAME: &'static str = #params_class_name;
            }
        });
    components
        .generated_private_impls
        .extend(quote! { #ctor_wrapper });

    // Create the constructor definitions
    let ctor_sig = components.gensym_const("CTOR_SIG");
    let method_sig = components.gensym_const("METHOD_SIG");
    let exported_method = components.gensym_const("EXPORT_JAVA_FN");
    let access = enumset_to_toks(&ctx, quote!(#nekojni_internal::MFlags), m_flags - MFlags::Static);
    components
        .generated_private_items
        .extend(quote_spanned! { sig_span =>
            const #ctor_sig: &'static str = #nekojni_internal::constcat_const!(
                "(",
                #(<#param_tys_elided as #nekojni::conversions::JavaConversionType>::JNI_TYPE,)*
                ")V",
            );
            const #method_sig: &'static str = #nekojni_internal::constcat_const!(
                "(",
                #(<#param_tys_elided as #nekojni::conversions::JavaConversionType>::JNI_TYPE,)*
                ")",
                <#ret_ty_elided as #nekojni_internal::ConstructorReturnTy<#synthetic_ty>>
                    ::RET_TY
            );
            pub const #exported_method: #nekojni_internal::exported_class::ExportedItem =
                #nekojni_internal::exported_class::ExportedItem::NativeConstructor {
                    flags: #access,
                    signature: #ctor_sig,
                    native_name: #wrapper_java_name,
                    native_signature: #method_sig,
                    super_signature: #nekojni_internal::constcat_const!(
                        <#ret_ty_elided as #nekojni_internal::ConstructorReturnTy<#synthetic_ty>>
                            ::SUPER_CTOR_SIGNATURE
                    ),
                };
        });
    components
        .exports
        .push(quote_spanned! { sig_span => __njni_priv::#exported_method });

    // Create a wrapper function for the constructor
    item.block.stmts.clear();
    item.sig.output = parse2(quote! { -> #nekojni::JniRef<#lt, #self_ty> })?;
    constructor_wrapper_java(ctx, components, item, attrs)?;

    Ok(true)
}

pub(crate) fn method_wrapper(
    ctx: &MacroCtx,
    components: &mut JavaClassCtx,
    item: &mut ImplItemMethod,
    is_synthetic: bool,
) -> Result<bool> {
    // process the method's attributes
    let mut attrs: FunctionAttrs = FromAttributes::from_attributes(&item.attrs)?;
    for attr in &mut item.attrs {
        if last_path_segment(&attr.path) == "jni" {
            mark_attribute_processed(attr);
        }
    }
    if !components.is_internal && !is_synthetic {
        attrs.check_internal_used()?;
    }
    attrs.acc_synthetic = is_synthetic;

    // process export to CLI tool
    if let Some(class_name) = &attrs.export_module_info {
        let class_name = parse_class_name(class_name)?;
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
    if attrs.init && attrs.constructor {
        error(Span::call_site(), "`#[jni(init)]` methods cannot be `#[jni(constructor)]`.")?;
    }
    if let Some(abi) = &item.sig.abi {
        let is_java = match &abi {
            // `extern "Java"`
            Abi { name: Some(str), .. } if str.value() == "Java" => true,
            // we treat bare `extern` as meaning `extern "Java"` in our macro
            Abi { name: None, .. } => true,
            // other extern declaration.
            _ => false,
        };
        if is_java {
            if attrs.init {
                error(Span::call_site(), "`#[jni(init)]` methods cannot be `extern \"Java\"`.")?;
            }
            if attrs.constructor {
                return constructor_wrapper_java(ctx, components, item, &attrs);
            } else {
                return method_wrapper_java(ctx, components, item, &attrs);
            }
        }
    }
    if attrs.constructor {
        constructor_wrapper_exported(ctx, components, item, &attrs)
    } else {
        method_wrapper_exported(ctx, components, item, &attrs)
    }
}
