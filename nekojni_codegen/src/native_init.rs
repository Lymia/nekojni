//! While we try to implement as much as we can in Rust, this is not always practical. For example,
//! this big blob of code implements the system that loads the native binary (if the choice is to
//! include it in a .jar).
//!
//! This is fairly complex JVM code written in pure bytecode. Good luck.

// TODO: Cleanup code for this. It'll probably be part of the `nekojni` crate and called
//       as a native method, just because complicated.

use crate::utils::*;
use nekojni_classfile::*;
use nekojni_signatures::*;
use std::collections::HashMap;

pub struct AutoloadPath {
    pub resource_prefix: String,
    pub native_library_name: String,
    pub temp_path_name: String,
    pub version: String,
}

pub fn generate(
    name: ClassName,
    classes: &mut HashMap<String, Vec<u8>>,
    autoload_prefix: Option<AutoloadPath>,
) {
    generate_initialization_class(name.clone(), classes, &autoload_prefix);
}

fn generate_initialization_class(
    name: ClassName,
    classes: &mut HashMap<String, Vec<u8>>,
    autoload_prefix: &Option<AutoloadPath>,
) {
    let mut writer = ClassWriter::new(CFlags::Public | CFlags::Synthetic | CFlags::Final, &name);
    writer.source_file("native_init.rs");

    // write fields
    writer.field(
        FFlags::Private | FFlags::Volatile | FFlags::Static | FFlags::Synthetic,
        "IS_INIT_COMPLETED",
        &Type::Boolean,
    );
    writer.field(
        FFlags::Private | FFlags::Volatile | FFlags::Static | FFlags::Synthetic,
        "IS_POISONED",
        &Type::Boolean,
    );

    const PLATFORM_WINDOWS: i32 = 1;
    const PLATFORM_MACOS: i32 = 2;
    const PLATFORM_LINUX: i32 = 3;

    const ARCH_X86: i32 = 4;
    const ARCH_X86_64: i32 = 5;
    const ARCH_AARCH64: i32 = 6;

    // private <init>() { }
    {
        let method = writer.method(
            MFlags::Private | MFlags::Synthetic,
            "<init>",
            &MethodSig::void(&[]),
        );
        method
            .code()
            .aload(0)
            .invokespecial(
                &ClassName::new(&["java", "lang"], "Object"),
                "<init>",
                &MethodSig::void(&[]),
            )
            .vreturn();
    }

    if let Some(autoload_prefix) = &autoload_prefix {
        // private static native void native_autoload_cleanup(
        //   String path, String version, String native_library_name,
        // )
        {
            writer.method(
                MFlags::Private | MFlags::Static | MFlags::Native | MFlags::Synthetic,
                "native_autoload_cleanup",
                &MethodSig::void(&[STRING, STRING, STRING]),
            );
        }

        // private static int getOperatingSystem()
        //
        // Returns the platform as an number.
        {
            let method = writer.method(
                MFlags::Private | MFlags::Static | MFlags::Synthetic,
                "getOperatingSystem",
                &MethodSig::new(Type::Int, &[]),
            );

            let mut code = method.code();

            // retrieve os name.
            get_prop(&mut code, "os.name");
            code.astore(0);
            str_prefix(&mut code, 0, "Your operating system is not supported!", &[
                (PLATFORM_WINDOWS, "Windows "),
                (PLATFORM_MACOS, "Mac "),
                (PLATFORM_LINUX, "Linux"),
            ]);
            code.ireturn();
        }

        // private static int getArchitecture()
        //
        // Returns the platform as an number.
        {
            let method = writer.method(
                MFlags::Private | MFlags::Static | MFlags::Synthetic,
                "getArchitecture",
                &MethodSig::new(Type::Int, &[]),
            );

            let mut code = method.code();

            // retrieve os name.
            get_prop(&mut code, "os.arch");
            code.astore(0);
            str_prefix(&mut code, 0, "Your CPU architecture is not supported!", &[
                (ARCH_X86, "x86"),
                (ARCH_X86, "i386"),
                (ARCH_X86, "i486"),
                (ARCH_X86, "i586"),
                (ARCH_X86, "i686"),
                (ARCH_X86_64, "amd64"),
                (ARCH_X86_64, "x86_64"),
                (ARCH_AARCH64, "aarch64"),
            ]);
            code.ireturn();
        }

        // private static String getTargetName(int platform, int arch)
        //
        // Returns the platform as an number.
        {
            let method = writer.method(
                MFlags::Private | MFlags::Static | MFlags::Synthetic,
                "getTargetName",
                &MethodSig::new(STRING, &[Type::Int, Type::Int]),
            );

            let arg_platform = 0;
            let arg_arch = 1;

            let mut code = method.code();

            // construct a new string builder
            new_builder(&mut code);

            // append the architecture name
            str_from_id(&mut code, arg_arch, &[
                (ARCH_X86, "x86"),
                (ARCH_X86_64, "x86_64"),
                (ARCH_AARCH64, "aarch64"),
            ]);
            str_append_chain(&mut code);

            // append the operating system name
            str_from_id(&mut code, arg_platform, &[
                (PLATFORM_WINDOWS, "-pc-windows-msvc"),
                (PLATFORM_MACOS, "-apple-darwin"),
                (PLATFORM_LINUX, "-unknown-linux-gnu"),
            ]);
            str_append_chain(&mut code);

            // return the platform as a string.
            code.invokevirtual(&STRINGBUILDER_CL, "toString", &MethodSig::new(STRING, &[]))
                .areturn();
        }

        // private static String getLibraryName(int platform, int arch, boolean isJarName)
        //
        // Returns the name of the shared library.
        {
            let method = writer.method(
                MFlags::Private | MFlags::Static | MFlags::Synthetic,
                "getLibraryName",
                &MethodSig::new(STRING, &[Type::Int, Type::Int, Type::Boolean]),
            );

            let arg_platform = 0;
            let arg_arch = 1;
            let arg_is_jar = 2;
            let var_builder = 3;

            let mut code = method.code();

            // construct a new string builder
            new_builder(&mut code);
            code.astore(var_builder);

            // prepend `lib` for macos and linux
            let next = LabelId::new();
            let append_lib = LabelId::new();
            code.iload(arg_platform)
                .iconst(PLATFORM_MACOS)
                .if_icmpeq(append_lib)
                .iload(arg_platform)
                .iconst(PLATFORM_LINUX)
                .if_icmpeq(append_lib)
                .goto(next)
                .label(append_lib)
                .aload(var_builder);
            str_append_const(&mut code, "lib");
            code.label(next);

            // append the library name
            let next = LabelId::new();
            let is_jar = LabelId::new();
            let versioned_name = format!(
                "{}-{}",
                &autoload_prefix.native_library_name, &autoload_prefix.version
            );
            code.aload(var_builder)
                .iload(arg_is_jar)
                .iconst(0)
                .if_icmpne(is_jar)
                .aconst_str(&versioned_name)
                .goto(next)
                .label(is_jar)
                .aconst_str(&autoload_prefix.native_library_name)
                .label(next);
            str_append_chain(&mut code);
            code.pop();

            // append the architecture name
            code.aload(var_builder);
            str_from_id(&mut code, arg_arch, &[
                (ARCH_X86, ".x86"),
                (ARCH_X86_64, ".x86_64"),
                (ARCH_AARCH64, ".aarch64"),
            ]);
            str_append(&mut code);

            // append the operating system extension
            code.aload(var_builder);
            str_from_id(&mut code, arg_platform, &[
                (PLATFORM_WINDOWS, ".dll"),
                (PLATFORM_MACOS, ".dylib"),
                (PLATFORM_LINUX, ".so"),
            ]);
            str_append(&mut code);

            // return the platform as a string.
            code.aload(var_builder)
                .invokevirtual(&STRINGBUILDER_CL, "toString", &MethodSig::new(STRING, &[]))
                .areturn();
        }

        // private static Path getLibraryStore()
        //
        // Returns the path to the location where the native binaries will be placed temporarily.
        {
            let method = writer.method(
                MFlags::Private | MFlags::Static | MFlags::Synthetic,
                "getLibraryStore",
                &MethodSig::new(PATH, &[]),
            );

            let mut code = method.code();

            // retrieve user home.
            code.aconst_str("user.home")
                .invokestatic(
                    &ClassName::new(&["java", "lang"], "System"),
                    "getProperty",
                    &MethodSig::new(STRING, &[STRING]),
                )
                .iconst(0)
                .anewarray(&STRING_CL)
                .invokestatic(
                    &ClassName::new(&["java", "nio", "file"], "Paths"),
                    "get",
                    &MethodSig::new(PATH, &[STRING, STRING.array()]),
                )
                .astore(0);

            // append the program name to the path
            code.aload(0)
                .aconst_str(&autoload_prefix.temp_path_name)
                .invokeinterface(&PATH_CL, "resolve", &MethodSig::new(PATH, &[STRING]))
                .astore(0);

            // create the paths if they do not exist
            let file_attribute =
                ClassName::new(&["java", "nio", "file", "attribute"], "FileAttribute");
            code.aload(0)
                .iconst(0)
                .anewarray(&file_attribute)
                .invokestatic(
                    &ClassName::new(&["java", "nio", "file"], "Files"),
                    "createDirectories",
                    &MethodSig::new(PATH, &[PATH, Type::from(file_attribute).array()]),
                )
                .pop();

            code.aload(0).areturn();
        }

        // private static synchronized void loadNativeLibrary()
        //
        // Loads an approprate native library from the .jar, if it can be found.
        {
            let method = writer.method(
                MFlags::Private | MFlags::Static | MFlags::Synchronized | MFlags::Synthetic,
                "loadNativeLibrary",
                &MethodSig::void(&[]),
            );

            let var_platform = 0;
            let var_arch = 1;
            let var_jar_artifact_name = 2;
            let var_cache_artifact_name = 3;
            let var_target_name = 4;
            let var_cache_path = 5;
            let var_resource_name = 6;
            let var_native_data = 7;
            let var_cache_dir = 8;

            let mut code = method.code();

            // find the operating system
            code.invokestatic(&name, "getOperatingSystem", &MethodSig::new(Type::Int, &[]))
                .istore(var_platform);

            // find the cpu arch
            code.invokestatic(&name, "getArchitecture", &MethodSig::new(Type::Int, &[]))
                .istore(var_arch);

            // get the target triple
            code.iload(var_platform)
                .iload(var_arch)
                .invokestatic(
                    &name,
                    "getTargetName",
                    &MethodSig::new(STRING, &[Type::Int, Type::Int]),
                )
                .astore(var_target_name);

            // find the native binary names
            code.iload(var_platform)
                .iload(var_arch)
                .iconst(0)
                .invokestatic(
                    &name,
                    "getLibraryName",
                    &MethodSig::new(STRING, &[Type::Int, Type::Int, Type::Boolean]),
                )
                .astore(var_cache_artifact_name)
                .iload(var_platform)
                .iload(var_arch)
                .iconst(1)
                .invokestatic(
                    &name,
                    "getLibraryName",
                    &MethodSig::new(STRING, &[Type::Int, Type::Int, Type::Boolean]),
                )
                .astore(var_jar_artifact_name);

            // find the temporary path
            code.invokestatic(&name, "getLibraryStore", &MethodSig::new(PATH, &[]))
                .astore(var_cache_dir);

            // find the full path of the binary
            code.aload(var_cache_dir)
                .aload(var_cache_artifact_name)
                .invokeinterface(&PATH_CL, "resolve", &MethodSig::new(PATH, &[STRING]))
                .astore(var_cache_path);

            // check if the library file exists, and skip to library loading if it does
            let link_option = ClassName::new(&["java", "nio", "file"], "LinkOption");
            let skip_write = LabelId::new();
            code.aload(var_cache_path)
                .iconst(0)
                .anewarray(&link_option)
                .invokestatic(
                    &ClassName::new(&["java", "nio", "file"], "Files"),
                    "exists",
                    &MethodSig::new(Type::Boolean, &[PATH, Type::from(link_option).array()]),
                )
                .iconst(0)
                .if_icmpne(skip_write);

            // calculate the resource name for the library
            new_builder(&mut code);
            str_append_chain_const(&mut code, &format!("{}/", &autoload_prefix.resource_prefix));
            str_append_chain_var(&mut code, var_target_name);
            str_append_chain_const(&mut code, "/");
            str_append_chain_var(&mut code, var_jar_artifact_name);
            code.invokevirtual(&STRINGBUILDER_CL, "toString", &MethodSig::new(STRING, &[]))
                .astore(var_resource_name);

            // load the resource data
            let next = LabelId::new();
            code.aconst_class(&name)
                .aload(var_resource_name)
                .invokevirtual(
                    &ClassName::new(&["java", "lang"], "Class"),
                    "getResourceAsStream",
                    &MethodSig::new(Type::class(&["java", "io"], "InputStream"), &[STRING]),
                )
                .dup()
                .ifnonnull(next);
            throw(&mut code, "Native binary for your platform was not found.");
            code.label(next)
                .invokevirtual(
                    &ClassName::new(&["java", "io"], "InputStream"),
                    "readAllBytes",
                    &MethodSig::new(Type::Byte.array(), &[]),
                )
                .astore(var_native_data);

            // write out the resource data to the target path
            let open_option = ClassName::new(&["java", "nio", "file"], "OpenOption");
            code.aload(var_cache_path)
                .aload(var_native_data)
                .iconst(0)
                .anewarray(&open_option)
                .invokestatic(
                    &ClassName::new(&["java", "nio", "file"], "Files"),
                    "write",
                    &MethodSig::new(PATH, &[
                        PATH,
                        Type::Byte.array(),
                        Type::from(open_option).array(),
                    ]),
                )
                .pop();

            // load the actual library
            code.label(skip_write)
                .aload(var_cache_path)
                .invokevirtual(&OBJECT_CL, "toString", &MethodSig::new(STRING, &[]))
                .invokestatic(
                    &ClassName::new(&["java", "lang"], "System"),
                    "loadLibrary",
                    &MethodSig::void(&[STRING]),
                );

            // run the native cleanup
            code.aload(var_cache_dir)
                .invokevirtual(&OBJECT_CL, "toString", &MethodSig::new(STRING, &[]))
                .aconst_str(&autoload_prefix.version)
                .aconst_str(&autoload_prefix.native_library_name)
                .invokestatic(
                    &name,
                    "native_autoload_cleanup",
                    &MethodSig::void(&[STRING, STRING, STRING]),
                );

            // return
            code.vreturn();
        }
    }

    // private static native void native_init()
    {
        writer.method(
            MFlags::Private | MFlags::Static | MFlags::Native | MFlags::Synthetic,
            "native_init",
            &MethodSig::void(&[]),
        );
    }

    // private static synchronized void checkInit()
    //
    // This function checks the IS_INIT_COMPLETED flag, and if it isn't set, it calls the actual
    // initialization code in this class.
    {
        let method = writer.method(
            MFlags::Private | MFlags::Static | MFlags::Synchronized | MFlags::Synthetic,
            "checkInit",
            &MethodSig::void(&[]),
        );

        let is_already_init = LabelId::new();
        let is_poisoned = LabelId::new();
        let mut code = method.code();
        code.getstatic(&name, "IS_POISONED", &Type::Boolean)
            .iconst(0)
            .if_icmpne(is_poisoned)
            .getstatic(&name, "IS_INIT_COMPLETED", &Type::Boolean)
            .iconst(0)
            .if_icmpne(is_already_init)
            .iconst(1)
            .putstatic(&name, "IS_POISONED", &Type::Boolean);
        if autoload_prefix.is_some() {
            code.invokestatic(&name, "loadNativeLibrary", &MethodSig::void(&[]));
        }
        code.invokestatic(&name, "native_init", &MethodSig::void(&[]))
            .iconst(1)
            .putstatic(&name, "IS_INIT_COMPLETED", &Type::Boolean)
            .iconst(0)
            .putstatic(&name, "IS_POISONED", &Type::Boolean)
            .label(is_already_init)
            .vreturn();
        code.label(is_poisoned);
        throw(
            &mut code,
            "Native library already failed to load, refusing to try again.",
        );
    }

    // public static void init()
    //
    // This is a simple wrapper around checkInit that gives a fastpath, since that method is
    // marked as synchronized.
    {
        let method = writer.method(
            MFlags::Public | MFlags::Static | MFlags::Synthetic,
            "init",
            &MethodSig::void(&[]),
        );

        let is_already_init = LabelId::new();
        method
            .code()
            .getstatic(&name, "IS_INIT_COMPLETED", &Type::Boolean)
            .iconst(0)
            .if_icmpne(is_already_init)
            .invokestatic(&name, "checkInit", &MethodSig::void(&[]))
            .label(is_already_init)
            .vreturn();
    }

    {
        let method = writer.method(
            MFlags::Public | MFlags::Static | MFlags::Synthetic,
            "main",
            &MethodSig::void(&[STRING.array()]),
        );

        method
            .code()
            .invokestatic(&name, "init", &MethodSig::void(&[]))
            .vreturn();
    }

    classes.insert(name.display_jni().to_string(), writer.into_vec());
}
