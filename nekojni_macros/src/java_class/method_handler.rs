use crate::{errors::*, java_class::JavaClassCtx, utils::*, MacroCtx};
use darling::FromAttributes;
use enumset::EnumSet;
use nekojni_classfile::MFlags;
use nekojni_signatures::{ClassName, MethodName};
use proc_macro2::{Span, TokenStream as SynTokenStream, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse2, spanned::Spanned, Attribute, FnArg, ImplItemMethod, Pat, ReturnType, Signature, Type,
};

// TODO: Rewrite methods to allow better interop between `JniRef` and `JniRefMut`.

#[derive(Debug, FromAttributes)]
#[darling(attributes(jni))]
pub(crate) struct FunctionAttrs {
    #[darling(default)]
    rename: Option<String>,

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
    pub(crate) fn check_internal_used(&self) -> Result<()> {
        if self.direct_export.is_some() || self.export_module_info.is_some() {
            error(Span::call_site(), "Attrs starting with `__njni_` are internal.")?;
        }
        Ok(())
    }

    pub(crate) fn parse_flags(&self, span: &impl Spanned) -> Result<EnumSet<MFlags>> {
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
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#ty>();
                            #nekojni_internal::check_jniref(promise);
                        });
                    self_mode = FuncSelfMode::EnvRef(ty);
                }
                FuncArg::EnvMut(ty) => {
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#ty>();
                            #nekojni_internal::check_jniref(promise);
                        });
                    self_mode = FuncSelfMode::EnvMut(ty);
                }
                FuncArg::ParamOwned(ty) => {
                    components
                        .generated_type_checks
                        .extend(quote_spanned! { ty.span() =>
                            let promise = #nekojni_internal::promise::<#ty>();
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

    // Java method name
    let java_name = match &attrs.rename {
        None => heck::AsLowerCamelCase(item.sig.ident.to_string()).to_string(),
        Some(name) => name.clone(),
    };

    // Setup important spans
    let item_span = item.span();
    let sig_span = item.sig.span();
    let output_span = item.sig.output.span();

    // Setup the parameter types.
    let (self_param, env, lt) = match self_mode {
        FuncSelfMode::EnvRef(_) => (
            quote_spanned!(sig_span => self: &#nekojni::JniRef<Self>),
            quote_spanned!(sig_span => #nekojni::JniRef::env(self)),
            quote_spanned!(sig_span => /* nothing */),
        ),
        FuncSelfMode::Static => (
            quote_spanned!(sig_span => env: impl #std::convert::AsRef<#nekojni::JniEnv<'env>>),
            quote_spanned!(sig_span => env),
            quote_spanned!(sig_span => <'env>),
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

    // Parse the type signature of the function.
    let mut param_types: Vec<_> = args.iter().map(FuncArgMode::ty).collect();

    let mut param_names = Vec::new();
    let mut param_names_java = Vec::new();
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
            let #java_name = <#ty as #nekojni::conversions::JavaConversion>::to_java_value(
                #in_arg, env,
            );
        });
        param_names.push(in_name);
        param_names_java.push(java_name);
    }

    let ret_ty = match &item.sig.output {
        ReturnType::Default => quote_spanned! { output_span => () },
        ReturnType::Type(_, ty) => quote_spanned! { output_span => #ty },
    };

    // Generate the body of the function
    let cache_field_name = components.gensym("cached_method_id");
    if self_mode == FuncSelfMode::Static {
        components.generated_cache.extend(quote_spanned! { item_span =>
            #cache_field_name: #nekojni_internal::OnceCache<#jni::objects::JStaticMethodID<'env>>,
        });
    } else {
        components
            .generated_cache
            .extend(quote_spanned! { item_span =>
                #cache_field_name: #nekojni_internal::OnceCache<#jni::objects::JMethodID<'env>>,
            });
    }

    let rust_class_name = item.sig.ident.to_string();
    let call_method = match self_mode {
        FuncSelfMode::EnvRef(_) => quote_spanned! { item_span =>
            let this = #nekojni::JniRef::this(self);
            let ret_val = env.call_method(
                this, #java_name, SIGNATURE, &[#(#param_names_java,)*],
            );
        },
        FuncSelfMode::Static => {
            let class_name = &components.class_name;
            quote_spanned! { item_span =>
                let ret_val = env.call_static_method(
                    #class_name, #java_name, SIGNATURE, &[#(#param_names_java,)*],
                );
            }
        }
        _ => unreachable!(),
    };
    let mut body = quote_spanned! { item_span =>
        const SIGNATURE: &'static str = #nekojni_internal::constcat!(
            "(",
            #(<#param_types as #nekojni::conversions::JavaConversionType>::JNI_TYPE,)*
            ")",
            <#ret_ty as #nekojni_internal::ImportReturnTy>::JNI_TYPE,
        );

        let env = #env;
        #param_conversion

        #call_method

        #nekojni_internal::ImportReturnTy::from_return_ty(
            #rust_class_name, env, ret_val.map_err(|x| x.into()),
        )
    };
    if self_mode == FuncSelfMode::Static {
        let wrapper_fn = components.gensym("wrapper_fn");
        body = quote_spanned! { item_span =>
            fn #wrapper_fn(env: #nekojni::JniEnv, #(#param_names: #param_types,)*) -> #ret_ty {
                #body
            }
            let env = *env.as_ref();
            #wrapper_fn(env, #(#param_names,)*)
        };
    }

    // Generate the function in the additional impl block
    let new_method = parse2::<ImplItemMethod>(quote_spanned! { item_span =>
        fn func #lt(#self_param, #(#param_names: #param_types,)*) -> #ret_ty {
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
    attrs: &FunctionAttrs,
) -> Result<bool> {
    item.sig.abi = None;

    let (self_mode, args) = process_method_args(ctx, components, &mut item.sig)?;

    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

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
    let m_flags = attrs.parse_flags(&item.sig)?;
    if m_flags.contains(MFlags::Abstract) {
        error(sig_span, "Concrete methods cannot be `abstract`.")?;
    }
    if !m_flags.contains(MFlags::Final) {
        if let FuncSelfMode::SelfMut | FuncSelfMode::EnvMut(_) = &self_mode {
            error(sig_span, "`open` methods cannot take `self` mutably.")?;
        }
    }

    // Parse the type signature of the function.
    let mut param_types: Vec<_> = args.iter().map(FuncArgMode::ty).collect();

    let mut param_names_java = Vec::new();
    let mut param_names_param = Vec::new();
    for arg in &args {
        param_names_java.push(components.gensym("java"));
        param_names_param.push(components.gensym("param"));
    }

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
    let (extra_param, extra_param_java, extract_ref) = match &self_mode {
        FuncSelfMode::SelfRef => {
            (quote! { id_param: u32, }, quote! { "I", }, quote_spanned! { sig_span =>
                <#nekojni::JniRef<Self> as #nekojni_internal::ExtractSelfParam<'_>>
                    ::extract(env, this, id_param)
            })
        }
        FuncSelfMode::SelfMut => {
            (quote! { id_param: u32, }, quote! { "I", }, quote_spanned! { sig_span =>
                <#nekojni::JniRefMut<Self> as #nekojni_internal::ExtractSelfParam<'_>>
                    ::extract(env, this, id_param)
            })
        }
        FuncSelfMode::EnvRef(ty) | FuncSelfMode::EnvMut(ty) => {
            (quote! { id_param: u32, }, quote! { "I", }, quote_spanned! { sig_span =>
                <#ty as #nekojni_internal::ExtractSelfParam<'_>>
                    ::extract(env, this, id_param)
            })
        }
        FuncSelfMode::Static => (quote!(), quote!(), quote!()),
    };
    let (self_param, mut fn_call_body, is_static) = match &self_mode {
        FuncSelfMode::SelfRef | FuncSelfMode::EnvRef(_) => (
            quote_spanned! { sig_span => this: #jni::sys::jobject },
            quote_spanned! { sig_span =>
                let this_ref = unsafe { #extract_ref };
                this_ref.#rust_name(#(#param_names_param,)*)
            },
            false,
        ),
        FuncSelfMode::SelfMut | FuncSelfMode::EnvMut(_) => (
            quote_spanned! { sig_span => this: #jni::sys::jobject },
            quote_spanned! { sig_span =>
                let mut this_ref = unsafe { #extract_ref };
                this_ref.#rust_name(#(#param_names_param,)*)
            },
            false,
        ),
        FuncSelfMode::Static => (
            quote_spanned! { sig_span => _: #jni::sys::jclass },
            quote_spanned! { sig_span =>
                Self::#rust_name(env, #(#param_names_param,)*)
            },
            true,
        ),
    };
    for (i, arg) in args.iter().enumerate() {
        let name_java = &param_names_java[i];
        let name_param = &param_names_param[i];
        match arg {
            FuncArgMode::ParamOwned(ty) => {
                fn_call_body = quote_spanned! { sig_span =>
                    let #name_param =
                        <#ty as #nekojni::conversions::JavaConversionOwned<'_>>
                            ::from_java(#name_java, env);
                    #fn_call_body
                };
            }
            FuncArgMode::ParamRef(ty) => {
                fn_call_body = quote_spanned! { sig_span =>
                    <#ty as #nekojni::conversions::JavaConversion<'_>>::from_java_ref(
                        #name_java, env, |#name_param| {
                            #fn_call_body
                        }
                    )
                };
            }
            FuncArgMode::ParamMut(ty) => {
                fn_call_body = quote_spanned! { sig_span =>
                    <#ty as #nekojni::conversions::JavaConversion<'_>>::from_java_mut(
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
    let wrapper_name = components.gensym(&format!("wrapper_{rust_name_str}"));
    let exported_method = components.gensym("EXPORTED_METHOD");

    let method_sig = components.gensym("METHOD_SIG");
    let method_sig_native = components.gensym("METHOD_SIG_NATIVE");

    let ret_ty = match &item.sig.output {
        ReturnType::Default => quote_spanned! { output_span => () },
        ReturnType::Type(_, ty) => quote_spanned! { output_span => #ty },
    };
    let self_ty = &components.self_ty;
    let access = enumset_to_toks(&ctx, quote!(#nekojni_internal::MFlags), m_flags);

    // Handle internal feature to directly export the Java_*_initialize function.
    let direct_export_attrs = if let Some(class_name) = &attrs.direct_export {
        let class_name = match ClassName::parse_java(class_name) {
            Ok(v) => v,
            Err(e) => error(Span::call_site(), format!("Could not parse class name: {e:?}"))?,
        };
        let method_name = MethodName::new(class_name, &rust_name_str);
        let method_name = method_name.display_jni_export().to_string();

        quote_spanned! { sig_span =>
            #[no_mangle]
            #[export_name = #method_name]
            pub // not technically an attr, but, this works anyway.
        }
    } else {
        quote!()
    };

    // Generate the actual code.
    components
        .generated_private_impls
        .extend(quote_spanned! { sig_span =>
            #direct_export_attrs
            extern "system" fn #wrapper_name<'env>(
                env: #jni::JNIEnv<'env>,
                #self_param,
                #extra_param
                #(#param_names_java: <#param_types as #nekojni::conversions::JavaConversionType>
                    ::JavaType,)*
            ) -> <#ret_ty as #nekojni_internal::MethodReturn>::ReturnTy {
                #nekojni_internal::catch_panic_jni::<#ret_ty, _>(env, |env| unsafe {
                    #fn_call_body
                })
            }
        });
    let java_sig_params = quote_spanned! { sig_span =>
        #(<#param_types as #nekojni::conversions::JavaConversionType>::JNI_TYPE,)*
    };
    let (method_sig, method_sig_native) = if is_static {
        (method_sig.clone(), method_sig.clone())
    } else {
        components
            .generated_private_items
            .extend(quote_spanned! { sig_span =>
                const #method_sig_native: &'static str = #nekojni_internal::constcat!(
                    "(",
                    #extra_param_java
                    #java_sig_params
                    ")",
                    <#ret_ty as #nekojni_internal::MethodReturn>::JNI_RETURN_TYPE
                );
            });
        (method_sig, method_sig_native)
    };
    components
        .generated_private_items
        .extend(quote_spanned! { sig_span =>
            const #method_sig: &'static str = #nekojni_internal::constcat!(
                "(",
                #java_sig_params
                ")",
                <#ret_ty as #nekojni_internal::MethodReturn>::JNI_RETURN_TYPE
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
                    fn_ptr: #self_ty::#wrapper_name as *mut #std::ffi::c_void,
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
    if item.sig.generics.params.iter().next().is_some() {
        // TODO: Allow lifetime parameters.
        error(item.sig.generics.span(), "`#[jni_exports]` may not contain generic functions.")?;
    }

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

    // process the method itself
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
            ClassName::new(&[&pkg_name], &version).display_jni_export(),
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
    if let Some(abi) = &item.sig.abi {
        if let Some(abi) = &abi.name {
            let abi = abi.value();
            if abi == "Java" {
                return method_wrapper_java(ctx, components, item, &attrs);
            }
        }
    }
    method_wrapper_exported(ctx, components, item, &attrs)
}
