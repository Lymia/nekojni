use nekojni_codegen::signatures::*;

fn make_test_types() -> Vec<(Type, &'static str, &'static str)> {
    vec![
        (Type::Byte, "byte", "B"),
        (Type::Short, "short", "S"),
        (Type::Int, "int", "I"),
        (Type::Long, "long", "J"),
        (Type::Float, "float", "F"),
        (Type::Double, "double", "D"),
        (Type::Boolean, "boolean", "Z"),
        (Type::Byte.array_dim(3), "byte[][][]", "[[[B"),
        (Type::new(BasicType::Int), "int", "I"),
        (Type::Boolean.array_dim(8), "boolean[][][][][][][][]", "[[[[[[[[Z"),
        (
            Type::class(&["java".into(), "lang".into()], "String"),
            "java.lang.String",
            "Ljava/lang/String;",
        ),
        (
            Type::class(&["java".into(), "lang".into()], "String")
                .array()
                .array(),
            "java.lang.String[][]",
            "[[Ljava/lang/String;",
        ),
        (
            Type::class(&["java".into(), "util".into()], "ArrayList"),
            "java.util.ArrayList",
            "Ljava/util/ArrayList;",
        ),
    ]
}

#[test]
fn test_display_types_java() {
    for (ty, java_ty, _) in make_test_types() {
        assert_eq!(&ty.display_java().to_string(), java_ty);
    }
}

#[test]
fn test_display_types_jni() {
    for (ty, _, jni_ty) in make_test_types() {
        assert_eq!(&ty.display_jni().to_string(), jni_ty);
    }
}

#[test]
fn test_parse_types_java() {
    for (ty, java_ty, _) in make_test_types() {
        assert_eq!(ty, Type::parse_java(java_ty).unwrap());
    }
}

#[test]
fn test_parse_types_jni() {
    for (ty, _, jni_ty) in make_test_types() {
        assert_eq!(ty, Type::parse_jni(jni_ty).unwrap());
    }
}
