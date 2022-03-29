use nekojni_signatures::*;

const TEST_SIGS: &[(
    MethodSig<'static>,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
)] = &[
    (
        MethodSig::void({
            const PARAMS: &'static [Type<'_>] = &[
                Type::Byte,
                Type::Short,
                Type::Int,
                Type::Long,
                Type::Float,
                Type::Double,
                Type::Boolean,
                Type::Char,
            ];
            PARAMS
        }),
        "(byte, short, int, long, float, double, boolean, char)",
        "(Byte, Short, Int, Long, Float, Double, Boolean, Char) => Unit",
        "(BSIJFDZC)V",
        "(BSIJFDZC)V",
    ),
    (
        MethodSig::new(Type::class(&["java", "lang"], "String"), &[]),
        "() -> java.lang.String",
        "() => java.lang.String",
        "()Ljava/lang/String;",
        "()Ljava/lang/String;",
    ),
    (
        MethodSig::new(Type::class(&["java", "lang"], "String").array(), {
            const PARAMS: &'static [Type<'_>] = &[
                Type::Byte,
                Type::Short.array(),
                Type::class(&["java", "lang"], "String"),
            ];
            PARAMS
        }),
        "(byte, short[], java.lang.String) -> java.lang.String[]",
        "(Byte, Array[Short], java.lang.String) => Array[java.lang.String]",
        "(B[SLjava/lang/String;)[Ljava/lang/String;",
        "(B[SLjava/lang/String;)[Ljava/lang/String;",
    ),
    (
        MethodSig::new(
            Type::generic_class(&["java", "util"], "HashMap", {
                const GENERICS: &'static [Type<'_>] = &[
                    Type::class(&["java", "lang"], "String"),
                    Type::generic_class(&["java", "util"], "ArrayList", {
                        const GENERICS: &'static [Type<'_>] = &[Type::class(&["java", "lang"], "String")];
                        GENERICS
                    }),
                ];
                GENERICS
            })
            .array(),
            {
                const PARAMS: &'static [Type<'_>] = &[Type::Int];
                PARAMS
            },
        ),
        "(int) -> java.util.HashMap<java.lang.String, java.util.ArrayList<java.lang.String>>[]",
        "(Int) => Array[java.util.HashMap[java.lang.String, java.util.ArrayList[java.lang.String]]]",
        "(I)[Ljava/util/HashMap;",
        "(I)[Ljava/util/HashMap<Ljava/lang/String;Ljava/util/ArrayList<Ljava/lang/String;>;>;",
    ),
];

#[test]
fn test_display_sigs_java() {
    for (sig, java_sig, _, _, _) in TEST_SIGS {
        assert_eq!(&sig.display_java().to_string(), java_sig);
    }
}

#[test]
fn test_display_sigs_scala() {
    for (sig, _, scala_sig, _, _) in TEST_SIGS {
        assert_eq!(&sig.display_scala().to_string(), scala_sig);
    }
}

#[test]
fn test_display_sigs_jni() {
    for (sig, _, _, jni_sig, _) in TEST_SIGS {
        assert_eq!(&sig.display_jni().to_string(), jni_sig);
    }
    for (sig, _, _, _, jni_sig_generic) in TEST_SIGS {
        assert_eq!(&sig.display_jni_generic().to_string(), jni_sig_generic);
    }
}

#[test]
fn test_parse_sigs_java() {
    for (sig, java_sig, _, _, _) in TEST_SIGS {
        assert_eq!(sig, &MethodSig::parse_java(java_sig).unwrap());
    }
}

#[test]
fn test_parse_sigs_scala() {
    for (sig, _, scala_sig, _, _) in TEST_SIGS {
        assert_eq!(sig, &MethodSig::parse_scala(scala_sig).unwrap());
    }
}

#[test]
fn test_parse_sigs_jni() {
    for (sig, _, _, _, jni_sig_generic) in TEST_SIGS {
        assert_eq!(sig, &MethodSig::parse_jni(jni_sig_generic).unwrap());
    }
}
