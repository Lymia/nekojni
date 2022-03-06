use std::borrow::Cow;

mod java_sigs;
mod jni_sigs;

/// A full reference to a Java method.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Method<'a> {
    pub class: ClassName<'a>,
    pub name: &'a str,
    pub sig: MethodSignature<'a>,
}

/// The signature of a given [`Method`].
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct MethodSignature<'a> {
    pub ret_ty: ReturnType<'a>,
    pub params: Cow<'a, [MethodParameter<'a>]>,
}

/// The return type of a given [`Method`].
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum ReturnType<'a> {
    Void,
    Ty(Type<'a>),
}

/// A parameter to a given [`Method`].
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct MethodParameter<'a> {
    pub ty: Type<'a>,
    pub name: &'a str,
}

/// A type signature to be used with JNI.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Type<'a> {
    pub basic_sig: BasicType<'a>,
    pub array_dim: usize,
}

/// A basic Java type.
///
/// This is a reference to a particular class or a non-array primitive. As arrays can be recursive,
/// [`TypeSignature`] is used to represent array dimensionality.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum BasicType<'a> {
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Boolean,
    Char,
    Class(ClassName<'a>),
}

/// The name of a Java class, including its full package path.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ClassName<'a> {
    pub package: Vec<&'a str>,
    pub name: &'a str,
}
