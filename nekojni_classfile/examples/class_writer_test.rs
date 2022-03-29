use nekojni_classfile::*;
use nekojni_signatures::*;
use std::fs::File;

fn main() {
    let mut class_file = ClassWriter::new(
        ClassAccessFlags::Public | ClassAccessFlags::Final,
        &ClassName::new(&["moe", "lymia", "princess"], "PrincessWriterTest"),
    );
    class_file.source_file("princess_writer.rs");
    class_file.field(
        FieldAccessFlags::Public | FieldAccessFlags::Volatile,
        "field0",
        &Type::Int,
    );
    class_file.field(
        FieldAccessFlags::Public | FieldAccessFlags::Static,
        "field1",
        &Type::generic_class(&["java", "util"], "ArrayList", &[Type::class(
            &["java", "lang"],
            "Integer",
        )
        .array()])
        .array(),
    );
    class_file.method(
        MethodAccessFlags::Private | MethodAccessFlags::Native,
        "method0",
        &MethodSig::new(Type::Int, &[Type::Int, Type::Int]),
    );
    class_file.method(
        MethodAccessFlags::Private | MethodAccessFlags::Native,
        "method1",
        &MethodSig::new(
            Type::generic_class(&["java", "util"], "ArrayList", &[Type::class(
                &["java", "lang"],
                "Integer",
            )]),
            &[Type::Int, Type::Int],
        ),
    );
    class_file
        .write(File::create("PrincessWriterTest.class").unwrap())
        .unwrap();
}
