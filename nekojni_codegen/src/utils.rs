use nekojni_classfile::MethodWriter;
use nekojni_signatures::*;

pub const OBJECT_CL: ClassName<'static> = ClassName::new(&["java", "lang"], "Object");
pub const OBJECT: Type<'static> = Type::class(&["java", "lang"], "Object");
pub const PATH_CL: ClassName<'static> = ClassName::new(&["java", "nio", "file"], "Path");
pub const PATH: Type<'static> = Type::class(&["java", "nio", "file"], "Path");
pub const STRING_CL: ClassName<'static> = ClassName::new(&["java", "lang"], "String");
pub const STRING: Type<'static> = Type::class(&["java", "lang"], "String");
pub const STRINGBUILDER_CL: ClassName<'static> = ClassName::new(&["java", "lang"], "StringBuilder");
pub const STRINGBUILDER: Type<'static> = Type::class(&["java", "lang"], "StringBuilder");

pub fn throw(code: &mut MethodWriter, message: &str) {
    code.new(&Type::class(&["java", "lang"], "RuntimeException"))
        .dup()
        .aconst_str(message)
        .invokespecial(
            &ClassName::new(&["java", "lang"], "RuntimeException"),
            "<init>",
            &MethodSig::void(&[STRING]),
        )
        .athrow();
}
pub fn new_builder(code: &mut MethodWriter) {
    code.new(&STRINGBUILDER)
        .dup()
        .invokespecial(&STRING_CL, "<init>", &MethodSig::void(&[]));
}
pub fn str_append(code: &mut MethodWriter, what: &str) {
    code.aconst_str(what)
        .invokevirtual(
            &STRINGBUILDER_CL,
            "append",
            &MethodSig::new(STRINGBUILDER, &[STRING]),
        )
        .pop();
}
