use nekojni_signatures::*;

const TEST_TYPES: &[(Type<'static>, &'static str, &'static str)] = &[
    (Type::Byte, "byte", "B"),
    (Type::Short, "short", "S"),
    (Type::Int, "int", "I"),
    (Type::Long, "long", "J"),
    (Type::Float, "float", "F"),
    (Type::Double, "double", "D"),
    (Type::Boolean, "boolean", "Z"),
    (Type::Byte.array_dim(3), "byte[][][]", "[[[B"),
    (Type::new(BasicType::Int), "int", "I"),
    (
        Type::Boolean.array_dim(8),
        "boolean[][][][][][][][]",
        "[[[[[[[[Z",
    ),
    (
        Type::class(&["java", "lang"], "String"),
        "java.lang.String",
        "Ljava/lang/String;",
    ),
    (
        Type::class(&["java", "lang"], "String").array().array(),
        "java.lang.String[][]",
        "[[Ljava/lang/String;",
    ),
    (
        Type::class(&["java", "util"], "ArrayList"),
        "java.util.ArrayList",
        "Ljava/util/ArrayList;",
    ),
];

#[test]
fn test_display_types_java() {
    for (ty, java_ty, _) in TEST_TYPES {
        assert_eq!(&ty.display_java().to_string(), java_ty);
    }
}

#[test]
fn test_display_types_jni() {
    for (ty, _, jni_ty) in TEST_TYPES {
        assert_eq!(&ty.display_jni().to_string(), jni_ty);
    }
}
