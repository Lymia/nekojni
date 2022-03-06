use crate::*;
use pest::error::*;
use pest_consume::{match_nodes, Parser};
use std::{
    fmt::{Display, Formatter, Write},
    ops::Deref,
};

#[derive(Parser)]
#[grammar = "java_signature.pest"]
struct JavaParser;
type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl JavaParser {
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
            [path(name), ty_array_braces(braces)..] => {
                let base = if name.package.is_empty() {
                    match name.name {
                        "byte" => Type::Byte,
                        "short" => Type::Short,
                        "int" => Type::Int,
                        "long" => Type::Long,
                        "float" => Type::Float,
                        "double" => Type::Double,
                        "boolean" => Type::Boolean,
                        "char" => Type::Char,
                        _ => Type::new(BasicType::Class(name)),
                    }
                } else {
                    Type::new(BasicType::Class(name))
                };
                base.array_dim(braces.count())
            },
            [path(name), ty_generics(_), ty_array_braces(braces)..] => {
                Type::new(BasicType::Class(name)).array_dim(braces.count())
            },
        ))
    }
    fn ty_generics(_input: Node) -> Result<()> {
        Ok(())
    }
    fn ty_array_braces(_input: Node) -> Result<()> {
        Ok(())
    }

    fn sig(input: Node) -> Result<MethodSig> {
        Ok(match_nodes!(input.children();
            [sig_param_list(params)] => {
                MethodSig::void_owned(&params)
            },
            [sig_param_list(params), ty(ret_ty)] => {
                MethodSig::new_owned(ret_ty, &params)
            },
        ))
    }
    fn sig_param_list(input: Node) -> Result<Vec<Type>> {
        Ok(match_nodes!(input.children();
            [ty(params)..] => params.collect(),
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
    pub fn parse_java(source: &'a str) -> Result<Self> {
        let inputs = JavaParser::parse(Rule::full_sig, source)?;
        let input = inputs.single()?;
        JavaParser::full_sig(input)
    }
}
impl<'a> Type<'a> {
    pub fn parse_java(source: &'a str) -> Result<Self> {
        let inputs = JavaParser::parse(Rule::full_ty, source)?;
        let input = inputs.single()?;
        JavaParser::full_ty(input)
    }
}
impl<'a> ClassName<'a> {
    pub fn parse_java(source: &'a str) -> Result<Self> {
        let inputs = JavaParser::parse(Rule::full_path, source)?;
        let input = inputs.single()?;
        JavaParser::full_path(input)
    }
}

fn display_params<'a>(sig: &'a MethodSig<'a>, f: &mut Formatter<'_>) -> std::fmt::Result {
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

struct DisplayMethodSignatureJava<'a>(&'a MethodSig<'a>);
impl<'a> Display for DisplayMethodSignatureJava<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        display_params(self.0, f)?;
        if self.0.ret_ty != ReturnType::Void {
            f.write_str(" -> ")?;
            Display::fmt(&self.0.ret_ty.display_java(), f)?;
        }
        Ok(())
    }
}
impl<'a> MethodSig<'a> {
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
