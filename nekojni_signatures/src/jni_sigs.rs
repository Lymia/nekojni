use crate::*;
use pest::error::*;
use pest_consume::{match_nodes, Parser};
use std::{
    fmt::{Display, Formatter, Write},
    ops::Deref,
};

#[derive(Parser)]
#[grammar = "jni_signature.pest"]
struct JniParser;
type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl JniParser {
    fn ident(input: Node) -> Result<&str> {
        Ok(input.as_str())
    }

    fn path(input: Node) -> Result<ClassName> {
        Ok(match_nodes!(input.children();
            [ident(names)..] => {
                let mut vec: Vec<_> = names.collect();
                let name = match vec.pop() {
                    Some(x) => x,
                    None => return Err(input.error("ClassName has no components??")),
                };
                ClassName::new_owned(&vec, name)
            },
        ))
    }

    fn ty(input: Node) -> Result<Type> {
        Ok(match_nodes!(input.children();
            [ty_array_head(braces).., ty_prim(prim)] => {
                let ty = match prim {
                    "B" => Type::Byte,
                    "S" => Type::Short,
                    "I" => Type::Int,
                    "J" => Type::Long,
                    "F" => Type::Float,
                    "D" => Type::Double,
                    "Z" => Type::Boolean,
                    "C" => Type::Char,
                    _ => unreachable!(),
                };
                ty.array_dim(braces.count())
            },
            [ty_array_head(braces).., ty_class(class)] =>
                Type::new(BasicType::Class(class)).array_dim(braces.count()),
        ))
    }
    fn ty_prim(input: Node) -> Result<&str> {
        Ok(input.as_str())
    }
    fn ty_class(input: Node) -> Result<ClassName> {
        Ok(match_nodes!(input.children();
            [path(path)] => path,
        ))
    }
    fn ty_array_head(_input: Node) -> Result<()> {
        Ok(())
    }

    fn sig(input: Node) -> Result<MethodSig> {
        Ok(match_nodes!(input.children();
            [ty(params)..] => {
                let params: Vec<_> = params.collect();
                MethodSig::void_owned(&params)
            },
            [ty(params).., sig_ret(ret_ty)] => {
                let params: Vec<_> = params.collect();
                MethodSig::new_owned(ret_ty, &params)
            },
        ))
    }
    fn sig_ret(input: Node) -> Result<Type> {
        Ok(match_nodes!(input.children();
            [ty(ty)] => ty,
        ))
    }

    fn full_ty(input: Node) -> Result<Type> {
        Ok(match_nodes!(input.children();
            [ty(ty), EOI(_)] => ty,
        ))
    }
    fn full_sig(input: Node) -> Result<MethodSig> {
        Ok(match_nodes!(input.children();
            [sig(sig), EOI(_)] => sig,
        ))
    }
    fn full_path(input: Node) -> Result<ClassName> {
        Ok(match_nodes!(input.children();
            [path(path), EOI(_)] => path,
        ))
    }
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }
}

impl<'a> MethodSig<'a> {
    /// Parses a method signature from a JNI format.
    pub fn parse_jni(source: &'a str) -> Result<Self> {
        let inputs = JniParser::parse(Rule::full_sig, source)?;
        let input = inputs.single()?;
        JniParser::full_sig(input)
    }
}
impl<'a> Type<'a> {
    /// Parses a type from a JNI format
    pub fn parse_jni(source: &'a str) -> Result<Self> {
        let inputs = JniParser::parse(Rule::full_ty, source)?;
        let input = inputs.single()?;
        JniParser::full_ty(input)
    }
}
impl<'a> ClassName<'a> {
    /// Parses a class name from a JNI format
    pub fn parse_jni(source: &'a str) -> Result<Self> {
        let inputs = JniParser::parse(Rule::full_path, source)?;
        let input = inputs.single()?;
        JniParser::full_path(input)
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
