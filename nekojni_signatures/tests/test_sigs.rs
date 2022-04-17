use nekojni_signatures::*;

fn make_test_sigs() -> Vec<(MethodSig, &'static str, &'static str)> {
    vec![
        (
            MethodSig::new(Type::Void, &[
                Type::Byte,
                Type::Short,
                Type::Int,
                Type::Long,
                Type::Float,
                Type::Double,
                Type::Boolean,
                Type::Char,
            ]),
            "(byte, short, int, long, float, double, boolean, char)",
            "(BSIJFDZC)V",
        ),
        (
            MethodSig::new(Type::class(vec!["java".into(), "lang".into()], "String"), &[]),
            "() -> java.lang.String",
            "()Ljava/lang/String;",
        ),
        (
            MethodSig::new(Type::class(vec!["java".into(), "lang".into()], "String").array(), &[
                Type::Byte,
                Type::Short.array(),
                Type::class(vec!["java".into(), "lang".into()], "String"),
            ]),
            "(byte, short[], java.lang.String) -> java.lang.String[]",
            "(B[SLjava/lang/String;)[Ljava/lang/String;",
        ),
    ]
}

#[test]
fn test_display_sigs_java() {
    for (sig, java_sig, _) in make_test_sigs() {
        assert_eq!(&sig.display_java().to_string(), java_sig);
    }
}

#[test]
fn test_display_sigs_jni() {
    for (sig, _, jni_sig) in make_test_sigs() {
        assert_eq!(&sig.display_jni().to_string(), jni_sig);
    }
}

#[test]
fn test_parse_sigs_java() {
    for (sig, java_sig, _) in make_test_sigs() {
        assert_eq!(sig, MethodSig::parse_java(java_sig).unwrap());
    }
}

#[test]
fn test_parse_sigs_jni() {
    for (sig, _, jni_sig) in make_test_sigs() {
        assert_eq!(sig, MethodSig::parse_jni(jni_sig).unwrap());
    }
}
