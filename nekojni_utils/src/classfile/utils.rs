use crate::{
    classfile::MethodWriter,
    signatures::{BasicType, Type},
};

pub fn type_stack_size(ty: &Type) -> u16 {
    if ty.array_dim != 0 {
        1
    } else {
        match &ty.basic_sig {
            BasicType::Byte => 1,
            BasicType::Short => 1,
            BasicType::Int => 1,
            BasicType::Long => 2,
            BasicType::Float => 1,
            BasicType::Double => 2,
            BasicType::Boolean => 1,
            BasicType::Char => 1,
            BasicType::Void => panic!("Void is not a valid stack type."),
            BasicType::Class(_) => 1,
        }
    }
}

pub fn push_param(code: &mut MethodWriter, id: u16, ty: &Type) -> u16 {
    if ty.array_dim != 0 {
        code.aload(id);
    } else {
        match &ty.basic_sig {
            BasicType::Byte
            | BasicType::Short
            | BasicType::Int
            | BasicType::Boolean
            | BasicType::Char => code.iload(id),
            BasicType::Long => code.lload(id),
            BasicType::Float => code.fload(id),
            BasicType::Double => code.dload(id),
            BasicType::Class(_) => code.aload(id),
            BasicType::Void => panic!("Void is not a valid parameter type."),
        };
    }
    type_stack_size(ty)
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
            BasicType::Void => code.vreturn(),
        };
    }
}
