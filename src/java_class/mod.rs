mod exports;

pub use exports::*;

/// A trait representing a Java class.
pub trait JavaClass {
    /// Contains the information needed to generate Java or Scala headers for this module.
    const EXPORTED: Option<ExportedClass> = None;
}
