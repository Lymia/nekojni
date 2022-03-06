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
#[allow(non_upper_case_globals)]
impl<'a> Type<'a> {
    pub const Byte: Type<'a> = Type::new(BasicType::Byte);
    pub const Short: Type<'a> = Type::new(BasicType::Short);
    pub const Int: Type<'a> = Type::new(BasicType::Int);
    pub const Long: Type<'a> = Type::new(BasicType::Long);
    pub const Float: Type<'a> = Type::new(BasicType::Float);
    pub const Double: Type<'a> = Type::new(BasicType::Double);
    pub const Boolean: Type<'a> = Type::new(BasicType::Boolean);
    pub const Char: Type<'a> = Type::new(BasicType::Char);

    /// Create a new type for a given basic type.
    pub const fn new(ty: BasicType<'a>) -> Self {
        Type {
            basic_sig: ty,
            array_dim: 0,
        }
    }

    /// Create a new class name.
    pub const fn class(package: &'a [&'a str], name: &'a str) -> Self {
        Type::new(BasicType::Class(ClassName::new(package, name)))
    }

    /// Create a new class name with a named package path.
    pub fn class_owned(package: &[&'a str], name: &'a str) -> Self {
        Type::new(BasicType::Class(ClassName::new_owned(package, name)))
    }

    /// Create a new type for an array.
    pub const fn array(mut self) -> Self {
        self.array_dim += 1;
        self
    }

    /// Create a new type for a multidimensional array.
    pub const fn array_dim(mut self, dims: usize) -> Self {
        self.array_dim += dims;
        self
    }
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
    pub package: Cow<'a, [&'a str]>,
    pub name: &'a str,
}
impl<'a> ClassName<'a> {
    /// Create a new class name.
    pub const fn new(package: &'a [&'a str], name: &'a str) -> Self {
        ClassName {
            package: Cow::Borrowed(package),
            name,
        }
    }

    /// Create a new class name with a named package path.
    pub fn new_owned(package: &[&'a str], name: &'a str) -> Self {
        ClassName {
            package: Cow::Owned(package.to_owned()),
            name,
        }
    }
}
