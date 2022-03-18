mod method_handler;

use darling::FromAttributes;
use crate::{errors::*, utils::*, MacroCtx};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse2, spanned::Spanned, ImplItem, ItemImpl, Type};
use nekojni_signatures::ClassName;

pub(crate) struct JavaClassCtx {
    class_name: TokenStream,
    settings: MacroArgs,
    sym_uid: usize,

    impl_extra_defs: TokenStream,
    wrapper_funcs: TokenStream,
}
impl JavaClassCtx {
    fn gensym(&mut self, prefix: &str) -> Ident {
        let ident = ident!("nekojni__{}_{}", prefix, self.sym_uid);
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
}

pub fn jni_export(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let ctx = MacroCtx::new()?;
    let mut item = parse2::<ItemImpl>(item)?;

    if item.generics.params.iter().next().is_some() {
        error(
            item.generics.span(),
            "`#[jni_exports]` may not be used with generic impls.",
        )?;
    }

    // Process macros on the impl block
    let args: MacroArgs = FromAttributes::from_attributes(&item.attrs)?;
    for attr in &mut item.attrs {
        if last_path_segment(&attr.path) == "jni" {
            mark_attribute_processed(attr);
        }
    }

    // Derive the class name and some other useful fields.
    let class_simple_name = match &args.java_name {
        None => match &*item.self_ty {
            Type::Path(ty) => last_path_segment(&ty.path),
            _ => error(
                item.self_ty.span(),
                "Cannot automatically retrieve java_name for this path. \
                Please use `#[jni(java_path = \"ExplicitName\")]`",
            )?,
        }
        Some(name) => name.clone(),
    };
    let package_str = args.package.clone().unwrap_or_else(String::new);
    let package_dot = if package_str.is_empty() { "." } else { "" };
    let class_name = format!("{package_str}{package_dot}{class_simple_name}");
    let class_name = match ClassName::parse_java(&class_name) {
        Ok(v) => v,
        Err(e) => error(attr.span(), format!("Could not parse class name: {e:?}"))?,
    };

    // Build the context.
    let class_name = crate::signatures::dump_class_name(&ctx, &class_name);
    let mut components = JavaClassCtx {
        class_name: class_name.clone(),
        settings: args,
        sym_uid: 0,
        impl_extra_defs: Default::default(),
        wrapper_funcs: Default::default(),
    };

    // Process methods in the impl block
    let mut errors = Error::empty();
    for item in &mut item.items {
        match item {
            ImplItem::Method(item) => {
                if let Err(e) = method_handler::method_wrapper(&ctx, &mut components, item) {
                    errors = errors.combine(e);
                }
            }
            _ => {}
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

    let impl_ty = &item.self_ty;

    Ok(quote! {
        #[allow(deprecated)]
        const _: () = {
            #item

            impl #nekojni_internal::JavaClassImpl for #impl_ty {
                const JAVA_TYPE: #nekojni::signatures::Type<'static> =
                    #nekojni::signatures::Type::new(
                        #nekojni::signatures::BasicType::Class(#class_name)
                    );
                const CODEGEN_INFO: Option<#nekojni::java_class::CodegenClass> = None;
                fn register_methods(&self, env: #jni::JNIEnv) -> #nekojni::Result<()> {
                    // TODO: register_methods
                    #nekojni::Result::Ok(())
                }
                fn default_ptr() -> &'static Self {
                    #nekojni_internal::default_ptr_fail()
                }
                fn create_jni_ref<'env>(
                    env: #jni::JNIEnv<'env>,
                    obj: #jni::objects::JObject<'env>,
                ) -> #nekojni::Result<#nekojni::JniRef<'env, Self>>
                    where Self: #nekojni::java_class::JavaClass
                {
                    #nekojni_internal::jni_ref::new_rust(env, obj)
                }
            }
            impl #nekojni::java_class::JavaClass for #impl_ty { }
            impl #nekojni_internal::RustContents for #impl_ty {
                const ID_FIELD: &'static str = "$$njit$id"; // TODO: Make this reactive to the type.
                fn get_manager() -> &'static #nekojni_internal::IdManager<
                    #nekojni_internal::parking_lot::RwLock<Self>
                > {
                    static MANAGER: #nekojni_internal::IdManager<
                        #nekojni_internal::parking_lot::RwLock<#impl_ty>
                    > = #nekojni_internal::IdManager::new();
                    &MANAGER
                }
            }

            ()
        };
    })
}
