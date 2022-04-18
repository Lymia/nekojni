use crate::{
    classfile::MethodWriter,
    signatures::{BasicType, Type},
};

pub fn push_param(code: &mut MethodWriter, id: u16, ty: &Type) -> u16 {
    if ty.array_dim != 0 {
        code.aload(id);
        1
    } else {
        match &ty.basic_sig {
            BasicType::Byte
            | BasicType::Short
            | BasicType::Int
            | BasicType::Boolean
            | BasicType::Char => {
                code.iload(id);
                2
            }
            BasicType::Long => {
                code.lload(id);
                1
            }
            BasicType::Float => {
                code.fload(id);
                1
            }
            BasicType::Double => {
                code.dload(id);
                2
            }
            BasicType::Class(_) => {
                code.aload(id);
                1
            }
            BasicType::Void => panic!("Void is not a valid parameter type."),
        }
    }
}
pub fn return_param(code: &mut MethodWriter, ty: &Type) {
    if ty.array_dim != 0 {
        code.areturn();
    } else {
        match &ty.basic_sig {
            BasicType::Byte
            | BasicType::Short
            | BasicType::Int
            | BasicType::Boolean
            | BasicType::Char => code.ireturn(),
            BasicType::Long => code.lreturn(),
            BasicType::Float => code.freturn(),
            BasicType::Double => code.dreturn(),
            BasicType::Class(_) => code.areturn(),
            BasicType::Void => code.areturn(),
        };
    }
}
