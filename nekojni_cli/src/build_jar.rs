use crate::native_loader::{EntryPointArch, EntryPointPlatform, ParsedBinary};
use nekojni::{
    __macro_internals::{
        exports::{jni_native_name, ExportedClass, ExportedItem},
        MARKER_STR,
    },
    *,
};
use nekojni_codegen::{signatures::ClassName, ClassData, Hasher, MFlags, NativeClassWrapper};
use std::collections::HashSet;

pub struct BuildJarOptions {
    pub main_bin: Option<String>,
    pub use_null_loader: bool,
}

pub fn make_jar_data(
    modules: &[ParsedBinary],
    build_jar_options: &BuildJarOptions,
) -> Result<ClassData> {
    let mut binaries: Vec<_> = modules.iter().collect();
    binaries.sort_by_key(|m| (&m.platform, &m.arch, m.path.file_name().unwrap()));
    let mut data = ClassData::new();

    // check for duplicated platform/arch combinations
    let mut used_combos = HashSet::new();
    for binary in &binaries {
        jni_assert!(
            used_combos.insert((binary.platform, binary.arch)),
            "Duplicate binary for platform {:?} with architecture {:?}.",
            binary.platform,
            binary.arch,
        );
    }

    // hash the input binaries
    let mut hasher = Hasher::new();
    hasher.update_str(MARKER_STR);
    for module in &binaries {
        let data = std::fs::read(&module.path)?;
        hasher.update(&[module.platform as u8, module.arch as u8]);
        hasher.update(&data.len().to_le_bytes());
        hasher.update(&data);
    }

    // find a binary that can be loaded on the current platform
    for binary_meta in &binaries {
        if let Some(binary) = binary_meta.load()? {
            // build the binary loader
            let loader_name = format!("moe/lymia/nekojni/rt/JarLoader_{:016x}", hasher.as_u64());
            if build_jar_options.use_null_loader {
                // the options required that we not use a loader
                data.add_null_loader(&loader_name);
            } else {
                // find the appropriate info metadata for the "main" module
                let info = if binary.info.len() == 1 {
                    binary.info[0]
                } else if let Some(main_bin) = &build_jar_options.main_bin {
                    let options: Vec<_> = binary
                        .info
                        .iter()
                        .filter(|x| x.crate_name == main_bin)
                        .collect();
                    jni_assert!(
                        options.len() == 0,
                        "More than one module in the native library matchs `--main-mod`."
                    );
                    options[0]
                } else {
                    jni_bail!(
                        "More than one module is present in the native library. \
                        Please use `--main-mod`"
                    );
                };

                // generate the actual loader class
                let loader = ClassName::parse_jni(info.init_class_name)?;
                let package_path = format!("{}/nekojni_native_libs", loader.package.join("/"));
                data.add_resource_loader(
                    &loader_name,
                    info.crate_name,
                    info.crate_version,
                    &package_path,
                );

                // copy the binaries to the jar resources
                for binary_meta in &binaries {
                    let (os_name, os_prefix, os_ext) = match binary_meta.platform {
                        // Though we call it -msvc, this could be compiled with gnu... Oh well.
                        EntryPointPlatform::Windows => ("pc-windows-msvc", "", ".dll"),
                        EntryPointPlatform::Macos => ("apple-darwin", "lib", ".dylib"),
                        EntryPointPlatform::Linux => ("unknown-linux-gnu", "lib", ".so"),
                    };
                    let arch_name = match binary_meta.arch {
                        EntryPointArch::X86 => "x86",
                        EntryPointArch::X86_64 => "x86_64",
                        EntryPointArch::AArch64 => "aarch64",
                    };

                    let binary_data = std::fs::read(&binary_meta.path)?;
                    data.add_resource(
                        &format!(
                            "{package_path}/{arch_name}-{os_name}/{os_prefix}{}{os_ext}",
                            info.crate_name,
                        ),
                        binary_data,
                    );
                }
            }

            // generate all classes to the .jar
            for module in &binary.info {
                data.add_module_loader(module.init_class_name);
                for class in module.class_info {
                    generate_class(&class.exported, &mut data, module.init_class_name);
                }
            }

            return Ok(data);
        }
    }

    // oh no :(
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
