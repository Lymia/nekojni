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

pub fn generate_initialization_class(
    name: ClassName,
    classes: &mut HashMap<String, Vec<u8>>,
    autoload_prefix: Option<AutoloadPath>,
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

    const PLATFORM_WINDOWS: i32 = 0;
    const PLATFORM_MACOS: i32 = 1;
    const PLATFORM_LINUX: i32 = 2;

    const ARCH_X86: i32 = 10;
    const ARCH_X86_64: i32 = 11;
    const ARCH_AARCH64: i32 = 12;

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
            code.aconst_str("os.name")
                .invokestatic(
                    &ClassName::new(&["java", "lang"], "System"),
                    "getProperty",
                    &MethodSig::new(STRING, &[STRING]),
                )
                .astore(0);

            for (id, plaf) in &[
                (PLATFORM_WINDOWS, "Windows "),
                (PLATFORM_MACOS, "Mac "),
                (PLATFORM_LINUX, "Linux"),
            ] {
                let next = LabelId::new();
                code.aload(0)
                    .aconst_str(plaf)
                    .invokevirtual(
                        &STRING_CL,
                        "startsWith",
                        &MethodSig::new(Type::Boolean, &[STRING]),
                    )
                    .iconst(0)
                    .if_icmpeq(next)
                    .iconst(*id)
                    .ireturn()
                    .label(next);
            }

            // throws an exception because oppps
            throw(&mut code, "Your operating system is not supported!");
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
            code.aconst_str("os.arch")
                .invokestatic(
                    &ClassName::new(&["java", "lang"], "System"),
                    "getProperty",
                    &MethodSig::new(STRING, &[STRING]),
                )
                .astore(0);

            for (id, arch) in &[
                (ARCH_X86, "x86"),
                (ARCH_X86, "i386"),
                (ARCH_X86, "i486"),
                (ARCH_X86, "i586"),
                (ARCH_X86, "i686"),
                (ARCH_X86_64, "amd64"),
                (ARCH_X86_64, "x86_64"),
                (ARCH_AARCH64, "aarch64"),
            ] {
                let next = LabelId::new();
                code.aload(0)
                    .aconst_str(arch)
                    .invokevirtual(
                        &STRING_CL,
                        "startsWith",
                        &MethodSig::new(Type::Boolean, &[STRING]),
                    )
                    .iconst(0)
                    .if_icmpeq(next)
                    .iconst(*id)
                    .ireturn()
                    .label(next);
            }

            // throws an exception because oppps
            throw(&mut code, "Your CPU architecture is not supported!");
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
            let var_builder = 2;

            let mut code = method.code();

            // construct a new string builder
            new_builder(&mut code);
            code.astore(2);

            // append the architecture name
            let next = LabelId::new();
            for (id, arch) in &[
                (ARCH_X86, "x86"),
                (ARCH_X86_64, "x86_64"),
                (ARCH_AARCH64, "aarch64"),
            ] {
                let cont = LabelId::new();
                code.iload(arg_arch)
                    .iconst(*id)
                    .if_icmpne(cont)
                    .aload(var_builder);
                str_append(&mut code, arch);
                code.goto(next)
                    .label(cont);
            }
            throw(&mut code, "internal error: bad arch code");
            code.label(next);

            // append the operating system name
            let next = LabelId::new();
            for (id, plaf) in &[
                (PLATFORM_WINDOWS, "-pc-windows-msvc"),
                (PLATFORM_MACOS, "-apple-darwin"),
                (PLATFORM_LINUX, "-unknown-linux-gnu"),
            ] {
                let cont = LabelId::new();
                code.iload(arg_platform)
                    .iconst(*id)
                    .if_icmpne(cont)
                    .aload(var_builder);
                str_append(&mut code, plaf);
                code.goto(next)
                    .label(cont);
            }
            throw(&mut code, "internal error: bad platform code");
            code.label(next);

            // return the platform as a string.
            code.aload(var_builder)
                .invokevirtual(&STRINGBUILDER_CL, "toString", &MethodSig::new(STRING, &[]))
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

            // append a platform-specific prefix
            let next = LabelId::new();
            for (id, plaf) in &[
                (PLATFORM_MACOS, "lib"),
                (PLATFORM_LINUX, "lib"),
            ] {
                let cont = LabelId::new();
                code.iload(arg_platform)
                    .iconst(*id)
                    .if_icmpne(cont)
                    .aload(var_builder);
                str_append(&mut code, plaf);
                code.goto(next)
                    .label(cont);
            }
            code.label(next);

            // append the library name
            let next = LabelId::new();
            let is_jar = LabelId::new();
            code.iload(arg_is_jar)
                .iconst(0)
                .if_icmpne(is_jar)
                .aload(var_builder);
            str_append(
                &mut code,
                &format!("{}-{}", &autoload_prefix.native_library_name, &autoload_prefix.version),
            );
            code.goto(next)
                .label(is_jar)
                .aload(var_builder);
            str_append(&mut code, &autoload_prefix.native_library_name);
            code.label(next);

            // append an archetecture suffix
            let next = LabelId::new();
            for (id, arch) in &[
                (ARCH_X86, ".x86"),
                (ARCH_X86_64, ".x86_64"),
                (ARCH_AARCH64, ".aarch64"),
            ] {
                let cont = LabelId::new();
                code.iload(arg_arch)
                    .iconst(*id)
                    .if_icmpne(cont)
                    .aload(var_builder);
                str_append(&mut code, arch);
                code.goto(next)
                    .label(cont);
            }
            throw(&mut code, "internal error: bad arch code");
            code.label(next);

            // append a platform-specific suffix
            let next = LabelId::new();
            for (id, plaf) in &[
                (PLATFORM_WINDOWS, ".dll"),
                (PLATFORM_MACOS, ".dylib"),
                (PLATFORM_LINUX, ".so"),
            ] {
                let cont = LabelId::new();
                code.iload(arg_platform)
                    .iconst(*id)
                    .if_icmpne(cont)
                    .aload(var_builder);
                str_append(&mut code, plaf);
                code.goto(next)
                    .label(cont);
            }
            throw(&mut code, "internal error: bad platform code");
            code.label(next);

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
                .anewarray(&STRING)
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
                Type::class(&["java", "nio", "file", "attribute"], "FileAttribute");
            code.aload(0)
                .iconst(0)
                .anewarray(&file_attribute)
                .invokestatic(
                    &ClassName::new(&["java", "nio", "file"], "Files"),
                    "createDirectories",
                    &MethodSig::new(PATH, &[PATH, file_attribute.array()])
                )
                .pop();

            code.aload(0)
                .areturn();
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
                .astore(var_cache_path);

            // find the full path of the binary
            code.aload(var_cache_path)
                .aload(var_cache_artifact_name)
                .invokeinterface(&PATH_CL, "resolve", &MethodSig::new(PATH, &[STRING]))
                .astore(var_cache_path);

            // check if the library file exists
            let link_option = Type::class(&["java", "nio", "file"], "LinkOption");
            let skip_write = LabelId::new();
            code.aload(var_cache_path)
                .iconst(0)
                .anewarray(&link_option)
                .invokestatic(
                    &ClassName::new(&["java", "nio", "file"], "Files"),
                    "exists",
                    &MethodSig::new(Type::Boolean, &[PATH, link_option.array()])
                )
                .iconst(0)
                .if_icmpne(skip_write);

            // copy the library file if it does not already exist
            // TODO: AAAA

            // load the actual library
            code.label(skip_write)
                .aload(var_cache_path)
                .invokevirtual(&OBJECT_CL, "toString", &MethodSig::new(STRING, &[]))
                .invokestatic(
                    &ClassName::new(&["java", "lang"], "System"),
                    "loadLibrary",
                    &MethodSig::void(&[STRING]),
                );

            // cleanup
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

    classes.insert(name.display_jni().to_string(), writer.into_vec());
}
