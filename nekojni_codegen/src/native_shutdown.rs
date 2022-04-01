use crate::utils::*;
use nekojni_classfile::*;
use nekojni_signatures::*;
use std::collections::HashMap;

pub fn generate_shutdown_handler(discriminator: &str) -> (String, Vec<u8>) {
    let class_name = format!("NekoJniShutdownHook$${}", discriminator);
    let name = ClassName::new(&["moe", "lymia", "nekojni", "generated_rt"], &class_name);

    let mut writer = ClassWriter::new(CFlags::Synthetic | CFlags::Final, &name);
    writer.extends(&ClassName::new(&["java", "lang"], "Thread"));
    writer.source_file("native_init.rs");

    // write fields
    writer.field(
        FFlags::Private | FFlags::Volatile | FFlags::Static | FFlags::Synthetic,
        "IS_SHUTDOWN",
        &Type::Boolean,
    );
    writer.field(
        FFlags::Private | FFlags::Volatile | FFlags::Static | FFlags::Synthetic,
        "IS_INSTALLED",
        &Type::Boolean,
    );

    // private <init>() { }
    {
        let method =
            writer.method(MFlags::Private | MFlags::Synthetic, "<init>", &MethodSig::void(&[]));
        method
            .code()
            .aload(0)
            .invokespecial(
                &ClassName::new(&["java", "lang"], "Thread"),
                "<init>",
                &MethodSig::void(&[]),
            )
            .vreturn();
    }

    // private static native void native_cleanup()
    {
        writer.method(
            MFlags::Private | MFlags::Static | MFlags::Native | MFlags::Synthetic,
            "native_shutdown",
            &MethodSig::void(&[]),
        );
    }

    // public synchronized void run()
    {
        let method = writer.method(
            MFlags::Public | MFlags::Final | MFlags::Synchronized,
            "run",
            &MethodSig::void(&[]),
        );

        let next = LabelId::new();
        let mut code = method.code();
        code.getstatic(&name, "IS_SHUTDOWN", &Type::Boolean)
            .iconst(0)
            .if_icmpeq(next);
        throw(&mut code, "native_shutdown already run!");
        code.label(next)
            .iconst(1)
            .putstatic(&name, "IS_SHUTDOWN", &Type::Boolean)
            .invokestatic(&name, "native_shutdown", &MethodSig::void(&[]))
            .vreturn();
    }

    // public static synchronized void install()
    {
        let method = writer.method(
            MFlags::Public | MFlags::Static | MFlags::Synthetic | MFlags::Synchronized,
            "install",
            &MethodSig::void(&[]),
        );

        let next = LabelId::new();
        let mut code = method.code();
        code.getstatic(&name, "IS_INSTALLED", &Type::Boolean)
            .iconst(0)
            .if_icmpeq(next);
        throw(&mut code, "install already run!");
        code.label(next)
            .iconst(1)
            .putstatic(&name, "IS_INSTALLED", &Type::Boolean)
            .invokestatic(
                &ClassName::new(&["java", "lang"], "Runtime"),
                "getRuntime",
                &MethodSig::new(Type::class(&["java", "lang"], "Runtime"), &[]),
            )
            .new(&name)
            .dup()
            .invokespecial(&name, "<init>", &MethodSig::void(&[]))
            .invokevirtual(
                &ClassName::new(&["java", "lang"], "Runtime"),
                "addShutdownHook",
                &MethodSig::void(&[Type::class(&["java", "lang"], "Thread")]),
            )
            .vreturn();
    }

    let name_str = name.display_jni().to_string();
    (name_str, writer.into_vec())
}
