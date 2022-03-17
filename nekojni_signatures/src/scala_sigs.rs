use crate::*;
use pest::error::*;
use pest_consume::{match_nodes, Parser};
use std::{
    fmt::{Display, Formatter, Write},
    ops::Deref,
};

// TODO: Handle Scala Any

#[derive(Parser)]
#[grammar = "scala_signature.pest"]
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
        let (name, generics) = match_nodes!(input.children();
            [path(name)] =>
                (name, StaticList::Borrowed(&[])),
            [path(name), ty_generics(generics)] =>
                (name, StaticList::Owned(generics)),
        );
        let mut base = if name.package.is_empty() {
            match name.name {
                // java primitives
                "byte" | "short" | "int" | "long" | "float" | "double" | "boolean" | "char" => {
                    return Err(input.error(format!(
                        "{} is a Java primitive type, and should not be used as a type name.",
                        name.name
                    )))
                }

                // scala primitives
                "Byte" | "Short" | "Int" | "Long" | "Float" | "Double" | "Boolean" | "Char" => {
                    let prim = match name.name {
                        "Byte" => Type::Byte,
                        "Short" => Type::Short,
                        "Int" => Type::Int,
                        "Long" => Type::Long,
                        "Float" => Type::Float,
                        "Double" => Type::Double,
                        "Boolean" => Type::Boolean,
                        "Char" => Type::Char,
                        _ => unreachable!(),
                    };
                    if !generics.is_empty() {
                        return Err(input.error("Primitive types are not generic."));
                    } else {
                        return Ok(prim);
                    }
                }
                "Array" => {
                    if generics.len() != 1 {
                        return Err(input.error("Array[T] must have exactly one type parameter."));
                    }
                    return Ok(generics[0].clone().array());
                }

                // random class in base package
                _ => Type::new(BasicType::Class(name)),
            }
        } else {
            Type::new(BasicType::Class(name))
        };

        // we have a normal class type
        base.generics = generics; // note: Scala allows primitives in generics.
        Ok(base)
    }
    fn ty_generics(input: Node) -> Result<Vec<Type>> {
        Ok(match_nodes!(input.children();
            [ty(params)..] => params.collect(),
        ))
    }

    fn sig(input: Node) -> Result<MethodSig> {
        Ok(match_nodes!(input.children();
            [sig_param_list(params), ty(ret_ty)] => {
                if let BasicType::Class(ref class) = ret_ty.basic_sig {
                    if class.package.is_empty() && class.name == "Unit" {
                        return Ok(MethodSig::void_owned(&params))
                    }
                }
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
    /// Parses a method signature from a Scala-like format. All class names must be fully
    /// qualified.
    ///
    /// The format used here "pretends" to be a Scala closure, with a format like `Int => Unit`
    /// or `(Int, Int) => Double`.
    pub fn parse_scala(source: &'a str) -> Result<Self> {
        let inputs = JavaParser::parse(Rule::full_sig, source)?;
        let input = inputs.single()?;
        JavaParser::full_sig(input)
    }
}
impl<'a> Type<'a> {
    /// Parses a type from a Scala-like format. All class names must be fully qualified.
    pub fn parse_scala(source: &'a str) -> Result<Self> {
        let inputs = JavaParser::parse(Rule::full_ty, source)?;
        let input = inputs.single()?;
        JavaParser::full_ty(input)
    }
}
impl<'a> ClassName<'a> {
    /// Parses a class name from a Scala-like format. All class names must be fully qualified.
    pub fn parse_scala(source: &'a str) -> Result<Self> {
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
        Display::fmt(&param.display_scala(), f)?;
        is_first = false;
    }
    f.write_str(")")?;
    Ok(())
}

struct DisplayMethodSignatureScala<'a>(&'a MethodSig<'a>);
impl<'a> Display for DisplayMethodSignatureScala<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        display_params(self.0, f)?;
        f.write_str(" => ")?;
        Display::fmt(&self.0.ret_ty.display_scala(), f)?;
        Ok(())
    }
}
impl<'a> MethodSig<'a> {
    /// Displays this object in Scala syntax.
    pub fn display_scala(&'a self) -> impl Display + 'a {
        DisplayMethodSignatureScala(self)
    }
}

struct DisplayReturnTypeScala<'a>(&'a ReturnType<'a>);
impl<'a> Display for DisplayReturnTypeScala<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ReturnType::Void => f.write_str("Unit"),
            ReturnType::Ty(ty) => Display::fmt(&ty.display_scala(), f),
        }
    }
}
impl<'a> ReturnType<'a> {
    /// Displays this object in Scala syntax.
    pub fn display_scala(&'a self) -> impl Display + 'a {
        DisplayReturnTypeScala(self)
    }
}

struct DisplayTypeScala<'a>(&'a Type<'a>);
impl<'a> Display for DisplayTypeScala<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.0.array_dim {
            f.write_str("Array[")?;
        }
        Display::fmt(&self.0.basic_sig.display_scala(), f)?;
        if !self.0.generics.is_empty() {
            f.write_char('[')?;
            let mut is_first = true;
            for generic in self.0.generics.as_slice() {
                if !is_first {
                    f.write_str(", ")?;
                }
                Display::fmt(&generic.display_scala(), f)?;
                is_first = false;
            }
            f.write_char(']')?;
        }
        for _ in 0..self.0.array_dim {
            f.write_str("]")?;
        }
        Ok(())
    }
}
impl<'a> Type<'a> {
    /// Displays this object in Scala syntax.
    pub fn display_scala(&'a self) -> impl Display + 'a {
        DisplayTypeScala(self)
    }
}

struct DisplayBasicTypeScala<'a>(&'a BasicType<'a>);
impl<'a> Display for DisplayBasicTypeScala<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            BasicType::Byte => f.write_str("Byte"),
            BasicType::Short => f.write_str("Short"),
            BasicType::Int => f.write_str("Int"),
            BasicType::Long => f.write_str("Long"),
            BasicType::Float => f.write_str("Float"),
            BasicType::Double => f.write_str("Double"),
            BasicType::Boolean => f.write_str("Boolean"),
            BasicType::Char => f.write_str("Char"),
            BasicType::Class(class) => Display::fmt(&class.display_scala(), f),
        }
    }
}
impl<'a> BasicType<'a> {
    /// Displays this object in Scala syntax.
    pub fn display_scala(&'a self) -> impl Display + 'a {
        DisplayBasicTypeScala(self)
    }
}

struct DisplayClassNameScala<'a>(&'a ClassName<'a>);
impl<'a> Display for DisplayClassNameScala<'a> {
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
    /// Displays this object in Scala syntax.
    pub fn display_scala(&'a self) -> impl Display + 'a {
        DisplayClassNameScala(self)
    }
}
