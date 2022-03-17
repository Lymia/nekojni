use nekojni_signatures::*;

const TEST_SIGS: &[(MethodSig<'static>, &'static str, &'static str)] = &[
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
        "(BSIJFDZC)V",
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
        "(B[SLjava/lang/String;)[Ljava/lang/String;",
    ),
];

#[test]
fn test_display_sigs_java() {
    for (sig, java_sig, _) in TEST_SIGS {
        assert_eq!(&sig.display_java().to_string(), java_sig);
    }
}

#[test]
fn test_display_sigs_jni() {
    for (sig, _, jni_sig) in TEST_SIGS {
        assert_eq!(&sig.display_jni().to_string(), jni_sig);
    }
}

#[test]
fn test_parse_sigs_java() {
    for (sig, java_sig, _) in TEST_SIGS {
        assert_eq!(sig, &MethodSig::parse_java(java_sig).unwrap());
    }
}

#[test]
fn test_parse_sigs_jni() {
    for (sig, _, jni_sig) in TEST_SIGS {
        assert_eq!(sig, &MethodSig::parse_jni(jni_sig).unwrap());
    }
}
