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

/// Represents something exported from a Java class defined in Rust.
///
/// This is primarily intended to allow code generation for the Java-side of the Rust bindings.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExportedItem {
    /// A constructor exported to JVM code from JNI.
    NativeConstructor {
        /// The Java type signature of the method.
        ///
        /// The return type must be `void` for this to work correctly.
        signature: MethodSig<'static>,
    },
    /// A method exported to JVM code from JNI.
    NativeMethod {
        /// The name of the method exposed publicly to Java code.
        java_name: &'static str,
        /// The name of the native method exported by the Rust module.
        ///
        /// If this is equal to `java_name`, no proxy function will be generated, and the native
        /// function will be directly exposed to Java code.
        native_fn_name: &'static str,
        /// The visibility of the method.
        visibility: JavaVisibility,
        /// The Java type signature of the method.
        signature: MethodSig<'static>,
    },
    /// A field stored in the Java class, and exposed to Rust code.
    ///
    /// This automatically generates the field on the Java side.
    JavaField {
        /// The name of the Java field.
        java_name: &'static str,
        /// The visibility of the Java field.
        visibility: JavaVisibility,
        /// The Java type of the field.
        ty: Type<'static>,
    },
}

/// A trait representing a Java class that may be exported via codegen.
#[derive(Debug)]
pub struct CodegenClass {
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
