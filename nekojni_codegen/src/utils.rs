use nekojni_classfile::{LabelId, MethodWriter};
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
    code.new(&ClassName::new(&["java", "lang"], "RuntimeException"))
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
    code.new(&STRINGBUILDER_CL).dup().invokespecial(
        &STRINGBUILDER_CL,
        "<init>",
        &MethodSig::void(&[]),
    );
}

pub fn str_append(code: &mut MethodWriter) {
    str_append_chain(code);
    code.pop();
}
pub fn str_append_const(code: &mut MethodWriter, what: &str) {
    str_append_chain_const(code, what);
    code.pop();
}
pub fn str_append_var(code: &mut MethodWriter, id: u16) {
    str_append_chain_var(code, id);
    code.pop();
}

pub fn str_append_chain(code: &mut MethodWriter) {
    code.invokevirtual(
        &STRINGBUILDER_CL,
        "append",
        &MethodSig::new(STRINGBUILDER, &[STRING]),
    );
}
pub fn str_append_chain_const(code: &mut MethodWriter, what: &str) {
    code.aconst_str(what);
    str_append_chain(code);
}
pub fn str_append_chain_var(code: &mut MethodWriter, id: u16) {
    code.aload(id);
    str_append_chain(code);
}

pub fn get_prop(code: &mut MethodWriter, prop: &str) {
    code.aconst_str(prop).invokestatic(
        &ClassName::new(&["java", "lang"], "System"),
        "getProperty",
        &MethodSig::new(STRING, &[STRING]),
    );
}

pub fn str_prefix(code: &mut MethodWriter, arg_str: u16, err: &str, prefixes: &[(i32, &str)]) {
    let cont = LabelId::new();
    for (id, prefix) in prefixes {
        let next = LabelId::new();
        code.aload(arg_str)
            .aconst_str(prefix)
            .invokevirtual(
                &STRING_CL,
                "startsWith",
                &MethodSig::new(Type::Boolean, &[STRING]),
            )
            .iconst(0)
            .if_icmpeq(next)
            .iconst(*id)
            .goto(cont)
            .label(next);
    }
    throw(code, err);
    code.label(cont);
}
pub fn str_from_id(code: &mut MethodWriter, arg_str: u16, prefixes: &[(i32, &str)]) {
    let next = LabelId::new();
    for (id, arch) in prefixes {
        let cont = LabelId::new();
        code.iload(arg_str)
            .iconst(*id)
            .if_icmpne(cont)
            .aconst_str(arch)
            .goto(next)
            .label(cont);
    }
    code.aconst_null().label(next);
}
