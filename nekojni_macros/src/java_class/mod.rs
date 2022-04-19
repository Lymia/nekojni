mod method_handler;

use crate::{errors::*, utils::*, MacroCtx};
use darling::FromAttributes;
use enumset::EnumSet;
use nekojni_utils::CFlags;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse2, spanned::Spanned, ImplItem, ItemImpl, Type};

pub(crate) struct JavaClassCtx {
    self_ty: TokenStream,

    class_name: String,
    settings: MacroArgs,
    sym_uid: usize,

    generated_impls: TokenStream,
    generated_private_impls: TokenStream,
    generated_private_items: TokenStream,
    generated_type_checks: TokenStream,

    exports: Vec<TokenStream>,
    native_methods: Vec<TokenStream>,

    is_internal: bool,
}
impl JavaClassCtx {
    fn gensym(&mut self, prefix: &str) -> Ident {
        let ident = ident!("__njni_{}_{}", prefix, self.sym_uid);
        self.sym_uid += 1;
        ident
    }
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(jni))]
pub(crate) struct MacroArgs {
    #[darling(default)]
    package: Option<String>,
    #[darling(default)]
    java_name: Option<String>,
    #[darling(default)]
    java_path: Option<String>,

    #[darling(default)]
    extends: Option<String>,
    #[darling(multiple)]
    implements: Vec<String>,

    #[darling(default, rename = "internal")]
    acc_internal: bool,
    #[darling(default, rename = "abstract")]
    acc_abstract: bool,
    #[darling(default, rename = "open")]
    acc_open: bool,
}
impl MacroArgs {
    pub(crate) fn parse_flags(&self, span: &impl Spanned) -> Result<EnumSet<CFlags>> {
        if !self.acc_open && self.acc_abstract {
            error(span.span(), "`abstract` classes must also be `open`.")?;
        }
        let mut flags = EnumSet::new();
        if !self.acc_internal {
            flags.insert(CFlags::Public);
        }
        if !self.acc_open {
            flags.insert(CFlags::Final);
        }
        if self.acc_abstract {
            flags.insert(CFlags::Abstract);
        }
        Ok(flags)
    }
}

fn jni_process_impl(
    attr: TokenStream,
    item: TokenStream,
    is_import: bool,
    is_internal: bool,
) -> Result<TokenStream> {
    let ctx = MacroCtx::new()?;
    let mut impl_block = parse2::<ItemImpl>(item)?;

    if impl_block.generics.params.iter().next().is_some() {
        error(
            impl_block.generics.span(),
            "`#[jni_exports]` may not be used with generic impls.",
        )?;
    }

    // Process macros on the impl block
    let args: MacroArgs = FromAttributes::from_attributes(&impl_block.attrs)?;
    for attr in &mut impl_block.attrs {
        if last_path_segment(&attr.path) == "jni" {
            mark_attribute_processed(attr);
        }
    }

    // Derive the class name and some other useful fields.
    let class_name = match &args.java_path {
        Some(path) => {
            if !args.package.is_none() {
                error(
                    attr.span(),
                    "`#[jni(java_path = ...)]` and `#[jni(package = ...)]` are mutually exclusive.",
                )?
            }
            path.clone()
        }
        None => {
            let class_simple_name = match &args.java_name {
                None => match &*impl_block.self_ty {
                    Type::Path(ty) => last_path_segment(&ty.path),
                    _ => error(
                        impl_block.self_ty.span(),
                        "Cannot automatically retrieve java_name for this path. \
                        Please use `#[jni(java_name = \"ExplicitName\")]`",
                    )?,
                },
                Some(name) => name.clone(),
            };
            let package_str = args.package.clone().unwrap_or_else(String::new);
            let package_dot = if package_str.is_empty() { "" } else { "." };
            format!("{package_str}{package_dot}{class_simple_name}")
        }
    };

    let class_name = parse_class_name(&class_name)?.display_jni().to_string();
    let cl_flags = args.parse_flags(&attr)?;
    let cl_id = if is_import { 0 } else { super::chain_next() };

    // Parse the supertypes.
    let extends_class = match &args.extends {
        Some(x) => Some(parse_class_name(&x)?.display_jni().to_string()),
        None => None,
    };
    let mut implements_classes = Vec::new();
    for class in &args.implements {
        implements_classes.push(parse_class_name(&class)?.display_jni().to_string());
    }

    // Build the context.
    let impl_ty = &impl_block.self_ty;
    let mut components = JavaClassCtx {
        self_ty: quote! { #impl_ty },
        class_name: class_name.clone(),
        settings: args,
        sym_uid: 0,
        generated_impls: Default::default(),
        generated_private_impls: Default::default(),
        generated_private_items: Default::default(),
        generated_type_checks: Default::default(),
        exports: Default::default(),
        native_methods: Default::default(),
        is_internal,
    };

    // Process methods in the impl block
    let mut errors = Error::empty();
    for item in std::mem::replace(&mut impl_block.items, Vec::new()) {
        match item {
            ImplItem::Method(mut method) => {
                match method_handler::method_wrapper(&ctx, &mut components, &mut method) {
                    Ok(true) => { /* remove this method */ }
                    Ok(false) => impl_block.items.push(ImplItem::Method(method)),
                    Err(e) => errors = errors.combine(e),
                }
            }
            item => impl_block.items.push(item),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    // Create the actual impl block
    let nekojni = &ctx.nekojni;
    let nekojni_internal = &ctx.internal;
    let std = &ctx.std;
    let jni = &ctx.jni;

    let generated_impls = &components.generated_impls;
    let generated_private_impls = &components.generated_private_impls;
    let generated_private_items = &components.generated_private_items;
    let generated_type_checks = &components.generated_type_checks;
    let exports = &components.exports;
    let native_methods = &components.native_methods;

    let create_ref = if is_import {
        quote! {
            fn default_ptr() -> &'static Self {
                &#impl_ty
            }
            fn create_jni_ref(
                env: #nekojni::JniEnv<'env>,
                obj: #jni::objects::JObject<'env>,
                id: Option<u32>,
            ) -> #nekojni::Result<#nekojni::JniRef<'env, Self>>
                where Self: #nekojni::objects::JavaClass<'env>
            {
                #nekojni_internal::jni_ref::new_wrapped(env, obj)
            }
        }
    } else {
        quote! {
            fn default_ptr() -> &'static Self {
                #nekojni_internal::default_ptr_fail()
            }
            fn create_jni_ref(
                env: #nekojni::JniEnv<'env>,
                obj: #jni::objects::JObject<'env>,
                id: Option<u32>,
            ) -> #nekojni::Result<#nekojni::JniRef<'env, Self>>
                where Self: #nekojni::objects::JavaClass<'env>
            {
                #nekojni_internal::jni_ref::new_rust(env, #class_name, obj, id)
            }
        }
    };

    let add_to_list_fn = if !is_import && !is_internal {
        let access = enumset_to_toks(&ctx, quote!(#nekojni_internal::CFlags), cl_flags);
        let extends = match extends_class {
            Some(name) => quote! { #std::option::Option::Some(#name) },
            None => quote! { #std::option::Option::None },
        };
        quote! {
            static CLASS_INFO: #nekojni_internal::JavaClassInfo =
                #nekojni_internal::JavaClassInfo {
                    name: #class_name,
                    exported: #nekojni_internal::exports::ExportedClass {
                        access: #access,
                        name: #class_name,
                        super_class: #extends,
                        implements: &[#(#implements_classes,)*],

                        id_field_name: "njni$$i",
                        late_init: &[],

                        exports: {
                            const LIST: &'static [#nekojni_internal::exports::ExportedItem] =
                                &[#(#exports,)*];
                            LIST
                        },
                        native_methods: {
                            const LIST: &'static [#nekojni_internal::exports::RustNativeMethod] =
                                &[#(#native_methods,)*];
                            LIST
                        },
                    },
                };
            fn append_to_list(classes: &crate::__njni_module_info::GatherClasses) {
                classes.0.borrow_mut().push(&CLASS_INFO)
            }
        }
    } else {
        quote! {
            fn append_to_list(classes: &crate::__njni_module_info::GatherClasses) {}
        }
    };
    let import_export_items = if !is_import {
        quote! {
            impl<'env> #nekojni_internal::RustContents<'env> for #impl_ty {
                const ID_FIELD: &'static str = "njni$$i";
            }
            impl<'a> #nekojni_internal::Registration<#cl_id>
                for crate::__njni_module_info::GatherClasses<'a>
            {
                #[inline(always)]
                fn run_chain_fwd(&self) {
                    use #nekojni_internal::{DerefRampChainA, DerefRampChainB};
                    append_to_list(self);
                    let helper = #nekojni_internal::DerefRamp::<{ #cl_id + 1 }, _>(self);
                    (&helper).run_chain_fwd();
                }
                #[inline(always)]
                fn run_chain_rev(&self) {
                    use #nekojni_internal::{DerefRampChainA, DerefRampChainB};
                    append_to_list(self);
                    let helper = #nekojni_internal::DerefRamp::<{ #cl_id - 1 }, _>(self);
                    (&helper).run_chain_rev();
                }
            }
            #add_to_list_fn
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #impl_block

        /// New code generated by nekojni.
        #[allow(deprecated)]
        const _: () = {
            impl #impl_ty {
                #generated_impls
            }
            impl<'env> #nekojni_internal::JavaClassImpl<'env> for #impl_ty {
                const INIT_ID: usize = #cl_id;
                const JNI_TYPE: &'static str = #class_name;
                const JNI_TYPE_SIG: &'static str =
                    #nekojni_internal::constcat_const!("L", #class_name, ";");

                #create_ref
            }
            impl<'env> #nekojni::objects::JavaClass<'env> for #impl_ty { }

            // Module used to seperate out private `self.*` functions
            #[allow(unused)]
            mod __njni_priv {
                use super::*;
                impl #impl_ty {
                    #generated_private_impls
                }
                #generated_private_items
            }

            #[allow(unused)]
            mod __njni_typeck {
                use super::*;
                impl #impl_ty {
                    fn __njni_macro_generated_type_checks() {
                        #generated_type_checks
                    }
                }
            }

            #import_export_items

            ()
        };
    })
}

pub fn jni_export(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    jni_process_impl(attr, item, false, false)
}
pub fn jni_export_internal(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    jni_process_impl(attr, item, false, true)
}
pub fn jni_import(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    jni_process_impl(attr, item, true, false)
}
