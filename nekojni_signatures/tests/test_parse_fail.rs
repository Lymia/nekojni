use nekojni_signatures::MethodSig;

const PARSE_FAIL_SIGS_JAVA: &[&str] = &[
    "(byte,",
    "()->",
    "() - > String",
    ",,,",
    ",)",
    "() -> java.util.ArrayList<java.lang.String><java.lang.String>",
    "() -> java.util.ArrayList<>",
    "() -> java.util.ArrayList[a]",
    "() -> java.util.ArrayList[]<java.lang.String>",
    "() -> java util ArrayList",
    "() -> java/util/ArrayList",
    "() -> Byte",
    "() -> Array<Int>",
    "() -> int<java.lang.String>",
    "() -> java.util.ArrayList<int>",
];
#[test]
fn test_parse_fail_sigs_java() {
    for sig in PARSE_FAIL_SIGS_JAVA {
        assert!(MethodSig::parse_java(sig).is_err(), "should not parse: {:?}", sig);
    }
}

// TODO: Finish this
