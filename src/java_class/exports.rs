use nekojni_signatures::{ClassName, MethodSig, Type};

/// The visibility of an exported Java type.
#[derive(Debug)]
pub enum JavaVisibility {
    /// Represents a `public` visibility
    Public,
    /// Represents a `protected` visibility.
    Protected,
    /// Represents a default visibility.
    PackagePrivate,
    /// Represents a `private` visibility.
    Private,
}

/// The attributes of a generated method or class.
#[derive(Debug)]
#[non_exhaustive]
pub struct MethodAttributes {
    /// The Java documentation of this method, formatted using the Javadoc or Scaladoc formatting.
    pub docs: &'static str,
    /// The visibility of a method.
    pub visibility: JavaVisibility,
}

/// Represents something exported from a Java class defined in Rust.
///
/// This is primarily intended to allow code generation for the Java-side of the Rust bindings.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExportedItem {
    /// A method exported to JVM code from JNI.
    NativeMethod {
        /// The name of the method exposed publicly to Java code.
        java_name: &'static str,
        /// The name of the native method exported by the Rust module.
        ///
        /// This is normally different from `java_name` to allow for some protection against
        /// mismatching type signatures, as this contains a hash of the type signature of the
        /// parameters of the Rust function when automatically derived.
        ///
        /// If this is equal to `java_name`, no proxy function will be generated, and the native
        /// function will be directly exposed to Java code.
        native_fn_name: &'static str,
        /// The attributes of the method.
        attributes: MethodAttributes,
        /// The Java type signature of the method.
        signature: MethodSig<'static>,
    },
    /// A field exported to JVM code from JNI.
    NativeField {
        /// The name of the field in Java.
        ///
        /// For Java code, this is converted into a pair of `getFoo` and `setFoo` methods. For
        /// Scala code, it generates a pair of `foo` and `foo_=` methods (allowing normal field
        /// access syntax).
        java_name: &'static str,
        /// The name of the native setter exported by the Rust module, or `None` if one shouldn't
        /// be generated.
        native_setter_name: Option<&'static str>,
        /// The name of the native getter exported by the Rust module, or `None` if one shouldn't
        /// be generated.
        native_getter_name: Option<&'static str>,
        /// The attributes of the generated setter. Ignored if no setter is generated.
        setter_attributes: MethodAttributes,
        /// The attributes of the generated getter. Ignored if no getter is generated.
        getter_attributes: MethodAttributes,
        /// The Java type of the field.
        ty: Type<'static>,
    },
    /// A field stored in the Java class, and exposed to Rust code.
    ///
    /// This automatically generates the field on the Java side.
    JavaField {
        /// The name of the Java field.
        java_name: &'static str,
        /// The attributes of the Java field.
        attributes: MethodAttributes,
        /// The Java type of the field.
        ty: Type<'static>,
    },
}

/// A trait representing a Java class that may be exported via codegen.
#[derive(Debug)]
pub struct CodegenClass {
    /// The Java documentation of this class, formatted using the Javadoc or Scaladoc formatting.
    pub docs: &'static str,
    /// The name of this class.
    pub name: ClassName<'static>,
    /// The visibility of this class.
    pub visibility: JavaVisibility,
    /// The name of this class' superclass, or `None` if there isn't one.
    pub super_class: Option<ClassName<'static>>,
    /// A list of interfaces this class implements.
    pub implements: Option<ClassName<'static>>,

    /// A function that returns a list of items exported into Java.
    pub get_exports: fn() -> Vec<ExportedItem>,
}
