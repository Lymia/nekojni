mod java_sigs;
mod jni_exports;
mod jni_sigs;

/// The signature of a given method.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct MethodSig {
    pub ret_ty: Type,
    pub params: Vec<Type>,
}
impl MethodSig {
    /// Creates a new method signature.
    pub fn new(ret_ty: Type, params: impl Into<Vec<Type>>) -> Self {
        MethodSig { ret_ty, params: params.into() }
    }
}

/// A type signature to be used with JNI.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Type {
    pub basic_sig: BasicType,
    pub array_dim: usize,
}
#[allow(non_upper_case_globals)]
impl Type {
    pub const Byte: Type = Type::new(BasicType::Byte);
    pub const Short: Type = Type::new(BasicType::Short);
    pub const Int: Type = Type::new(BasicType::Int);
    pub const Long: Type = Type::new(BasicType::Long);
    pub const Float: Type = Type::new(BasicType::Float);
    pub const Double: Type = Type::new(BasicType::Double);
    pub const Boolean: Type = Type::new(BasicType::Boolean);
    pub const Char: Type = Type::new(BasicType::Char);
    pub const Void: Type = Type::new(BasicType::Void);

    /// Create a new type for a given basic type.
    pub const fn new(ty: BasicType) -> Self {
        Type { basic_sig: ty, array_dim: 0 }
    }

    /// Create a new class name.
    pub fn class(package: impl Into<Vec<String>>, name: impl Into<String>) -> Self {
        Type::new(BasicType::Class(ClassName::new(package, name)))
    }

    /// Create a new type for an array.
    pub fn array(mut self) -> Self {
        self.array_dim += 1;
        self
    }

    /// Create a new type for a multidimensional array.
    pub fn array_dim(mut self, dims: usize) -> Self {
        self.array_dim += dims;
        self
    }
}
impl From<ClassName> for Type {
    fn from(cn: ClassName) -> Self {
        Type::new(BasicType::Class(cn))
    }
}

/// A basic Java type.
///
/// This is a reference to a particular class or a non-array primitive. As arrays can be recursive,
/// [`Type`] is used to represent array dimensionality.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum BasicType {
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Boolean,
    Char,
    Void,
    Class(ClassName),
}

/// The name of a Java class, including its full package path.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ClassName {
    pub package: Vec<String>,
    pub name: String,
}
impl ClassName {
    /// Create a new class name.
    pub fn new(package: impl Into<Vec<String>>, name: impl Into<String>) -> Self {
        ClassName { package: package.into(), name: name.into() }
    }
}

/// The name of a Java method.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct MethodName {
    pub class: ClassName,
    pub name: String,
}
impl MethodName {
    /// Create a new class name.
    pub fn new(class: ClassName, name: impl Into<String>) -> Self {
        MethodName { class, name: name.into() }
    }
}
