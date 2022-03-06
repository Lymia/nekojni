use nekojni_signatures::*;

const TEST_TYPES: &[(Type<'static>, &'static str, &'static str)] = &[
    (Type::new(BasicType::Byte), "byte", "B"),
    (Type::Byte.array_dim(3), "byte[][][]", "[[[B"),
    (Type::new(BasicType::Int), "int", "I"),
    (Type::new(BasicType::Boolean), "boolean", "Z"),
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
];

#[test]
fn test_types_java() {
    for (ty, java_ty, _) in TEST_TYPES {
        assert_eq!(&ty.display_java().to_string(), java_ty);
    }
}

#[test]
fn test_types_jni() {
    for (ty, _, jni_ty) in TEST_TYPES {
        assert_eq!(&ty.display_jni().to_string(), jni_ty);
    }
}
