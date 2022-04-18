use crate::native_loader::ParsedBinary;
use nekojni::{
    __macro_internals::{
        exports::{jni_native_name, ExportedClass, ExportedItem},
        MARKER_STR,
    },
    *,
};
use nekojni_codegen::{ClassData, Hasher, MFlags, NativeClassWrapper};

pub struct BuildJarOptions {}

pub fn make_jar_data(
    modules: &[ParsedBinary],
    build_jar_options: &BuildJarOptions,
) -> Result<ClassData> {
    let mut binaries: Vec<_> = modules.iter().collect();
    binaries.sort_by_key(|m| (&m.platform, &m.arch, m.path.file_name().unwrap()));

    let mut data = ClassData::new();

    let mut hasher = Hasher::new();
    hasher.update_str(MARKER_STR);
    for module in &binaries {
        let data = std::fs::read(&module.path)?;
        hasher.update(&[module.platform as u8, module.arch as u8]);
        hasher.update(&data.len().to_le_bytes());
        hasher.update(&data);
    }

    data.add_null_loader(&format!("moe/lymia/nekojni/rt/JarLoader_{:016x}", hasher.as_u64(),));

    for binary in binaries {
        if let Some(binary) = binary.load()? {
            for module in &binary.info {
                data.add_module_loader(module.init_class_name);
                for class in module.class_info {
                    generate_class(&class.exported, &mut data, module.init_class_name);
                }
            }

            return Ok(data);
        }
    }

    jni_bail!("No modules could be loaded!")
}

fn generate_class(data: &ExportedClass, class_data: &mut ClassData, init_class: &str) {
    let mut class = NativeClassWrapper::new(
        data.access,
        &data.name,
        match &data.super_class {
            None => "java/lang/Object",
            Some(v) => v,
        },
        data.id_field_name,
    );
    class.generate_init(init_class);
    for class_name in data.implements {
        class.implements(class_name);
    }

    for exports in data.exports {
        match exports {
            ExportedItem::NativeConstructor {
                flags,
                signature,
                native_name,
                native_signature,
                super_signature,
            } => {
                class.export_constructor(
                    *flags,
                    &signature,
                    &jni_native_name(native_name, true),
                    &native_signature,
                    &super_signature,
                    data.late_init,
                );
            }
            ExportedItem::NativeMethodWrapper {
                flags,
                name,
                signature,
                native_name,
                native_signature,
                has_id_param,
            } => {
                class.export_native_wrapper(
                    *flags,
                    name,
                    signature,
                    &jni_native_name(native_name, flags.contains(MFlags::Static)),
                    native_signature,
                    *has_id_param,
                );
            }
            ExportedItem::JavaField { flags, name, field } => {
                class.export_field(*flags, name, &field);
            }
        }
    }
    for method in data.native_methods {
        class.export_native(
            &jni_native_name(method.name, method.is_static),
            &method.sig,
            method.is_static,
        );
    }

    class_data.add_exported_class(class);
}
