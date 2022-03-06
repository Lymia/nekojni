use nekojni_signatures::*;

const TEST_SIGS: &[(MethodSig<'static>, &'static str, &'static str)] = &[(
    MethodSig::void({
        const PARAMS: &'static [MethodParam<'static>] = &[
            MethodParam::new(Type::Byte, "p0"),
            MethodParam::new(Type::Short, "p1"),
            MethodParam::new(Type::Int, "p2"),
            MethodParam::new(Type::Long, "p3"),
            MethodParam::new(Type::Float, "p4"),
            MethodParam::new(Type::Double, "p5"),
            MethodParam::new(Type::Boolean, "p6"),
            MethodParam::new(Type::Char, "p7"),
        ];
        PARAMS
    }),
    "void method(byte p0, short p1, int p2, long p3, float p4, double p5, boolean p6, char p7)",
    "(BSIJFDZC)V",
),
    (
        MethodSig::new(Type::class(&["java", "lang"], "String").array(), {
            const PARAMS: &'static [MethodParam<'static>] = &[
                MethodParam::new(Type::Byte, "p0"),
                MethodParam::new(Type::Short.array(), "p1"),
                MethodParam::new(Type::class(&["java", "lang"], "String"), "p2"),
            ];
            PARAMS
        }),
        "java.lang.String[] method(byte p0, short[] p1, java.lang.String p2)",
        "(B[SLjava/lang/String;)[Ljava/lang/String;",
    )];

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
