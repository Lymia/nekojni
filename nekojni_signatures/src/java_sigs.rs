use crate::*;
use pest_derive::*;
use std::{
    fmt::{Display, Formatter, Write},
    ops::Deref,
};

#[derive(Parser)]
#[grammar = "java_signature.pest"]
struct JavaParser;

struct DisplayMethodJava<'a>(&'a Method<'a>);
impl<'a> Display for DisplayMethodJava<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.sig.ret_ty.display_java(), f)?;
        f.write_char(' ')?;
        Display::fmt(&self.0.class.display_java(), f)?;
        f.write_char('.')?;
        f.write_str(&self.0.name)?;
        display_params(&self.0.sig, f)?;
        Ok(())
    }
}
impl<'a> Method<'a> {
    /// Displays this object in Java syntax.
    pub fn display_java(&'a self) -> impl Display + 'a {
        DisplayMethodJava(self)
    }
}

fn display_params<'a>(sig: &'a MethodSignature<'a>, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_char('(')?;
    let mut is_first = true;
    for param in sig.params.deref() {
        if !is_first {
            f.write_str(", ")?;
        }
        Display::fmt(&param.display_java(), f)?;
        is_first = false;
    }
    f.write_str(")")?;
    Ok(())
}

struct DisplayMethodSignatureJava<'a>(&'a MethodSignature<'a>);
impl<'a> Display for DisplayMethodSignatureJava<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.ret_ty.display_java(), f)?;
        f.write_str(" method")?;
        display_params(self.0, f)?;
        Ok(())
    }
}
impl<'a> MethodSignature<'a> {
    /// Displays this object in Java syntax.
    pub fn display_java(&'a self) -> impl Display + 'a {
        DisplayMethodSignatureJava(self)
    }
}

struct DisplayReturnTypeJava<'a>(&'a ReturnType<'a>);
impl<'a> Display for DisplayReturnTypeJava<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ReturnType::Void => f.write_str("void"),
            ReturnType::Ty(ty) => Display::fmt(&ty.display_java(), f),
        }
    }
}
impl<'a> ReturnType<'a> {
    /// Displays this object in Java syntax.
    pub fn display_java(&'a self) -> impl Display + 'a {
        DisplayReturnTypeJava(self)
    }
}

struct DisplayMethodParameterJava<'a>(&'a MethodParameter<'a>);
impl<'a> Display for DisplayMethodParameterJava<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.ty.display_java(), f)?;
        f.write_char(' ')?;
        f.write_str(&self.0.name)?;
        Ok(())
    }
}
impl<'a> MethodParameter<'a> {
    /// Displays this object in Java syntax.
    pub fn display_java(&'a self) -> impl Display + 'a {
        DisplayMethodParameterJava(self)
    }
}

struct DisplayTypeJava<'a>(&'a Type<'a>);
impl<'a> Display for DisplayTypeJava<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.basic_sig.display_java(), f)?;
        for _ in 0..self.0.array_dim {
            f.write_str("[]")?;
        }
        Ok(())
    }
}
impl<'a> Type<'a> {
    /// Displays this object in Java syntax.
    pub fn display_java(&'a self) -> impl Display + 'a {
        DisplayTypeJava(self)
    }
}

struct DisplayBasicTypeJava<'a>(&'a BasicType<'a>);
impl<'a> Display for DisplayBasicTypeJava<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            BasicType::Byte => f.write_str("byte"),
            BasicType::Short => f.write_str("short"),
            BasicType::Int => f.write_str("int"),
            BasicType::Long => f.write_str("long"),
            BasicType::Float => f.write_str("float"),
            BasicType::Double => f.write_str("double"),
            BasicType::Boolean => f.write_str("boolean"),
            BasicType::Char => f.write_str("char"),
            BasicType::Class(class) => Display::fmt(&class.display_java(), f),
        }
    }
}
impl<'a> BasicType<'a> {
    /// Displays this object in Java syntax.
    pub fn display_java(&'a self) -> impl Display + 'a {
        DisplayBasicTypeJava(self)
    }
}

struct DisplayClassNameJava<'a>(&'a ClassName<'a>);
impl<'a> Display for DisplayClassNameJava<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for pkg in self.0.package.deref() {
            f.write_str(pkg)?;
            f.write_char('.')?;
        }
        f.write_str(self.0.name)?;
        Ok(())
    }
}
impl<'a> ClassName<'a> {
    /// Displays this object in Java syntax.
    pub fn display_java(&'a self) -> impl Display + 'a {
        DisplayClassNameJava(self)
    }
}
