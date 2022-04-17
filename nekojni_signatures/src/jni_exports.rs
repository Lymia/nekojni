use crate::*;
use std::{
    fmt::{Display, Formatter, Write},
    ops::Deref,
};

struct DisplayString<'a>(&'a str);
impl<'a> Display for DisplayString<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for char in self.0.chars() {
            match char {
                'a'..='z' | 'A'..='Z' | '0'..='9' => f.write_char(char)?,
                '_' => f.write_str("_1")?,
                ';' => f.write_str("_2")?,
                '[' => f.write_str("_3")?,
                _ => write!(f, "_0{:04x}", char as u32)?,
            }
        }
        Ok(())
    }
}

struct DisplayJniClassName<'a>(&'a ClassName<'a>);
impl<'a> Display for DisplayJniClassName<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for package_name in self.0.package.deref() {
            Display::fmt(&DisplayString(package_name), f)?;
            f.write_char('_')?;
        }
        Display::fmt(&DisplayString(self.0.name), f)?;
        Ok(())
    }
}
impl<'a> ClassName<'a> {
    /// Displays this object as an JNI export symbol name.
    pub fn display_jni_export(&'a self) -> impl Display + 'a {
        DisplayJniClassName(self)
    }
}

struct DisplayJniExport<'a>(&'a MethodName<'a>);
impl<'a> Display for DisplayJniExport<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Java_")?;
        Display::fmt(&self.0.class.display_jni_export(), f)?;
        f.write_char('_')?;
        Display::fmt(&DisplayString(self.0.name), f)?;
        Ok(())
    }
}
impl<'a> MethodName<'a> {
    /// Displays this object as an JNI export symbol name.
    pub fn display_jni_export(&'a self) -> impl Display + 'a {
        DisplayJniExport(self)
    }
}
