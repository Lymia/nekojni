use nekojni_signatures::*;

const TEST_TYPES: &[(Type<'static>, &'static str, &'static str, &'static str)] = &[
    (Type::Byte, "byte", "Byte", "B"),
    (Type::Short, "short", "Short", "S"),
    (Type::Int, "int", "Int", "I"),
    (Type::Long, "long", "Long", "J"),
    (Type::Float, "float", "Float", "F"),
    (Type::Double, "double", "Double", "D"),
    (Type::Boolean, "boolean", "Boolean", "Z"),
    (
        Type::Byte.array_dim(3),
        "byte[][][]",
        "Array[Array[Array[Byte]]]",
        "[[[B",
    ),
    (Type::new(BasicType::Int), "int", "Int", "I"),
    (
        Type::Boolean.array_dim(8),
        "boolean[][][][][][][][]",
        "Array[Array[Array[Array[Array[Array[Array[Array[Boolean]]]]]]]]",
        "[[[[[[[[Z",
    ),
    (
        Type::class(&["java", "lang"], "String"),
        "java.lang.String",
        "java.lang.String",
        "Ljava/lang/String;",
    ),
    (
        Type::class(&["java", "lang"], "String").array().array(),
        "java.lang.String[][]",
        "Array[Array[java.lang.String]]",
        "[[Ljava/lang/String;",
    ),
    (
        Type::class(&["java", "util"], "ArrayList"),
        "java.util.ArrayList",
        "java.util.ArrayList",
        "Ljava/util/ArrayList;",
    ),
];
const TEST_TYPES_GENERIC: &[(Type<'static>, &'static str, &'static str, &'static str)] = &[
    (
        Type::generic_class(&["java", "util"], "ArrayList", {
            const GENERICS: &'static [Type<'static>] = &[Type::class(&["java", "lang"], "String")];
            GENERICS
        }),
        "java.util.ArrayList<java.lang.String>",
        "java.util.ArrayList[java.lang.String]",
        "Ljava/util/ArrayList;",
    ),
    (
        Type::generic_class(&["java", "util"], "HashMap", {
            const GENERICS: &'static [Type<'static>] = &[
                Type::class(&["java", "lang"], "String"),
                Type::class(&["java", "lang"], "String"),
            ];
            GENERICS
        })
        .array(),
        "java.util.HashMap<java.lang.String, java.lang.String>[]",
        "Array[java.util.HashMap[java.lang.String, java.lang.String]]",
        "[Ljava/util/HashMap;",
    ),
];

#[test]
fn test_display_types_java() {
    for (ty, java_ty, _, _) in TEST_TYPES {
        assert_eq!(&ty.display_java().to_string(), java_ty);
    }
    for (ty, java_ty, _, _) in TEST_TYPES_GENERIC {
        assert_eq!(&ty.display_java().to_string(), java_ty);
    }
}

#[test]
fn test_display_types_scala() {
    for (ty, _, scala_ty, _) in TEST_TYPES {
        assert_eq!(&ty.display_scala().to_string(), scala_ty);
    }
    for (ty, _, scala_ty, _) in TEST_TYPES_GENERIC {
        assert_eq!(&ty.display_scala().to_string(), scala_ty);
    }
}

#[test]
fn test_display_types_jni() {
    for (ty, _, _, jni_ty) in TEST_TYPES {
        assert_eq!(&ty.display_jni().to_string(), jni_ty);
    }
    for (ty, _, _, jni_ty) in TEST_TYPES_GENERIC {
        assert_eq!(&ty.display_jni().to_string(), jni_ty);
    }
}

#[test]
fn test_parse_types_java() {
    for (ty, java_ty, _, _) in TEST_TYPES {
        assert_eq!(ty, &Type::parse_java(java_ty).unwrap());
    }
    for (ty, java_ty, _, _) in TEST_TYPES_GENERIC {
        assert_eq!(ty, &Type::parse_java(java_ty).unwrap());
    }
}

#[test]
fn test_parse_types_scala() {
    for (ty, _, scala_ty, _) in TEST_TYPES {
        assert_eq!(ty, &Type::parse_scala(scala_ty).unwrap());
    }
    for (ty, _, scala_ty, _) in TEST_TYPES_GENERIC {
        assert_eq!(ty, &Type::parse_scala(scala_ty).unwrap());
    }
}

#[test]
fn test_parse_types_jni() {
    for (ty, _, _, jni_ty) in TEST_TYPES {
        assert_eq!(ty, &Type::parse_jni(jni_ty).unwrap());
    }
}
