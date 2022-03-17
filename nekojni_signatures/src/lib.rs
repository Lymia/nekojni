#![deny(unused_must_use)]

mod java_sigs;
mod jni_exports;
mod jni_sigs;
mod scala_sigs;
mod static_list;

pub use static_list::StaticList;

/// The signature of a given method.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct MethodSig<'a> {
    pub ret_ty: ReturnType<'a>,
    pub params: StaticList<'a, Type<'a>>,
}
impl<'a> MethodSig<'a> {
    /// Creates a new method signature.
    pub const fn new(ret_ty: Type<'a>, params: &'a [Type<'a>]) -> Self {
        MethodSig {
            ret_ty: ReturnType::Ty(ret_ty),
            params: StaticList::Borrowed(params),
        }
    }

    /// Creates a new method signature with an owned parameter list.
    pub fn new_owned(ret_ty: Type<'a>, params: &[Type<'a>]) -> Self {
        MethodSig {
            ret_ty: ReturnType::Ty(ret_ty),
            params: StaticList::Owned(params.to_vec()),
        }
    }

    /// Creates a new method signature that returns void.
    pub const fn void(params: &'a [Type<'a>]) -> Self {
        MethodSig {
            ret_ty: ReturnType::Void,
            params: StaticList::Borrowed(params),
        }
    }

    /// Creates a new method signature that returns void with an owned parameter list.
    pub fn void_owned(params: &[Type<'a>]) -> Self {
        MethodSig {
            ret_ty: ReturnType::Void,
            params: StaticList::Owned(params.to_vec()),
        }
    }
}

/// The return type of a given [`MethodSig`].
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum ReturnType<'a> {
    Void,
    Ty(Type<'a>),
}

/// A type signature to be used with JNI.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Type<'a> {
    pub basic_sig: BasicType<'a>,
    pub array_dim: usize,
    pub generics: StaticList<'a, Type<'a>>,
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
            generics: StaticList::Borrowed(&[]),
        }
    }

    /// Create a new type for a given basic type with generics.
    pub const fn new_generic(ty: BasicType<'a>, generics: &'a [Type<'a>]) -> Self {
        Type {
            basic_sig: ty,
            array_dim: 0,
            generics: StaticList::Borrowed(generics),
        }
    }

    /// Create a new type for a given basic type with an owned generics list.
    pub fn new_generic_owned(ty: BasicType<'a>, generics: &[Type<'a>]) -> Self {
        Type {
            basic_sig: ty,
            array_dim: 0,
            generics: StaticList::Owned(generics.to_vec()),
        }
    }

    /// Create a new class name.
    pub const fn class(package: &'a [&'a str], name: &'a str) -> Self {
        Type::new(BasicType::Class(ClassName::new(package, name)))
    }

    /// Create a new class name with an owned package path.
    pub fn class_owned(package: &[&'a str], name: &'a str) -> Self {
        Type::new(BasicType::Class(ClassName::new_owned(package, name)))
    }

    /// Create a new class name with generics.
    pub const fn generic_class(
        package: &'a [&'a str],
        name: &'a str,
        generics: &'a [Type<'a>],
    ) -> Self {
        Type::new_generic(BasicType::Class(ClassName::new(package, name)), generics)
    }

    /// Create a new class name with an owned package path and generics list.
    pub fn generic_class_owned(package: &[&'a str], name: &'a str, generics: &[Type<'a>]) -> Self {
        Type::new_generic_owned(
            BasicType::Class(ClassName::new_owned(package, name)),
            generics,
        )
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

    /// Returns whether this is a primitive type.
    pub fn is_primitive(&self) -> bool {
        self.array_dim != 0 || self.basic_sig.is_primitive()
    }
}

/// A basic Java type.
///
/// This is a reference to a particular class or a non-array primitive. As arrays can be recursive,
/// [`Type`] is used to represent array dimensionality.
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
impl<'a> BasicType<'a> {
    /// Returns whether this is a primitive type.
    pub fn is_primitive(&self) -> bool {
        match self {
            BasicType::Byte => true,
            BasicType::Short => true,
            BasicType::Int => true,
            BasicType::Long => true,
            BasicType::Float => true,
            BasicType::Double => true,
            BasicType::Boolean => true,
            BasicType::Char => true,
            BasicType::Class(_) => false,
        }
    }
}

/// The name of a Java class, including its full package path.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ClassName<'a> {
    pub package: StaticList<'a, &'a str>,
    pub name: &'a str,
}
impl<'a> ClassName<'a> {
    /// Create a new class name.
    pub const fn new(package: &'a [&'a str], name: &'a str) -> Self {
        ClassName {
            package: StaticList::Borrowed(package),
            name,
        }
    }

    /// Create a new class name with an owned package path.
    pub fn new_owned(package: &[&'a str], name: &'a str) -> Self {
        ClassName {
            package: StaticList::Owned(package.to_vec()),
            name,
        }
    }
}

/// The name of a Java method.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct MethodName<'a> {
    pub class: ClassName<'a>,
    pub name: &'a str,
}
impl<'a> MethodName<'a> {
    /// Create a new class name.
    pub const fn new(class: ClassName<'a>, name: &'a str) -> Self {
        MethodName { class, name }
    }
}
