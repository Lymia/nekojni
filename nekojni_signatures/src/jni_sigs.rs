use crate::*;
use pest_derive::*;
use std::{
    fmt::{Display, Formatter, Write},
    ops::Deref,
};

#[derive(Parser)]
#[grammar = "jni_signature.pest"]
struct JavaParser;

struct DisplayMethodJni<'a>(&'a Method<'a>);
impl<'a> Display for DisplayMethodJni<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.class.display_jni(), f)?;
        f.write_char('.')?;
        f.write_str(&self.0.name)?;
        Display::fmt(&self.0.sig.display_jni(), f)?;
        Ok(())
    }
}
impl<'a> Method<'a> {
    /// Displays this object in JNI descriptor syntax.
    pub fn display_jni(&'a self) -> impl Display + 'a {
        DisplayMethodJni(self)
    }
}

struct DisplayMethodSignatureJni<'a>(&'a MethodSig<'a>);
impl<'a> Display for DisplayMethodSignatureJni<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;
        for param in self.0.params.deref() {
            Display::fmt(&param.display_jni(), f)?;
        }
        f.write_str(")")?;
        Display::fmt(&self.0.ret_ty.display_jni(), f)?;
        Ok(())
    }
}
impl<'a> MethodSig<'a> {
    /// Displays this object in JNI descriptor syntax.
    pub fn display_jni(&'a self) -> impl Display + 'a {
        DisplayMethodSignatureJni(self)
    }
}

struct DisplayReturnTypeJni<'a>(&'a ReturnType<'a>);
impl<'a> Display for DisplayReturnTypeJni<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ReturnType::Void => f.write_str("V"),
            ReturnType::Ty(ty) => Display::fmt(&ty.display_jni(), f),
        }
    }
}
impl<'a> ReturnType<'a> {
    /// Displays this object in JNI descriptor syntax.
    pub fn display_jni(&'a self) -> impl Display + 'a {
        DisplayReturnTypeJni(self)
    }
}

struct DisplayMethodParameterJni<'a>(&'a MethodParam<'a>);
impl<'a> Display for DisplayMethodParameterJni<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.ty.display_jni(), f)
    }
}
impl<'a> MethodParam<'a> {
    /// Displays this object in JNI descriptor syntax.
    pub fn display_jni(&'a self) -> impl Display + 'a {
        DisplayMethodParameterJni(self)
    }
}

struct DisplayTypeJni<'a>(&'a Type<'a>);
impl<'a> Display for DisplayTypeJni<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.0.array_dim {
            f.write_str("[")?;
        }
        Display::fmt(&self.0.basic_sig.display_jni(), f)?;
        Ok(())
    }
}
impl<'a> Type<'a> {
    /// Displays this object in JNI descriptor syntax.
    pub fn display_jni(&'a self) -> impl Display + 'a {
        DisplayTypeJni(self)
    }
}

struct DisplayBasicTypeJni<'a>(&'a BasicType<'a>);
impl<'a> Display for DisplayBasicTypeJni<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            BasicType::Byte => f.write_str("B"),
            BasicType::Short => f.write_str("S"),
            BasicType::Int => f.write_str("I"),
            BasicType::Long => f.write_str("J"),
            BasicType::Float => f.write_str("F"),
            BasicType::Double => f.write_str("D"),
            BasicType::Boolean => f.write_str("Z"),
            BasicType::Char => f.write_str("C"),
            BasicType::Class(class) => Display::fmt(&class.display_jni(), f),
        }
    }
}
impl<'a> BasicType<'a> {
    /// Displays this object in JNI descriptor syntax.
    pub fn display_jni(&'a self) -> impl Display + 'a {
        DisplayBasicTypeJni(self)
    }
}

struct DisplayClassNameJni<'a>(&'a ClassName<'a>);
impl<'a> Display for DisplayClassNameJni<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('L')?;
        for pkg in self.0.package.deref() {
            f.write_str(pkg)?;
            f.write_char('/')?;
        }
        f.write_str(self.0.name)?;
        f.write_char(';')?;
        Ok(())
    }
}
impl<'a> ClassName<'a> {
    /// Displays this object in JNI descriptor syntax.
    pub fn display_jni(&'a self) -> impl Display + 'a {
        DisplayClassNameJni(self)
    }
}
