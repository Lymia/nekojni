use crate::classfile::PoolWriter;
use byteorder::{WriteBytesExt, BE};
use std::{
    fmt::Debug,
    io::{Cursor, Error, Write},
};

/// An attribute that may be written to a Java class file.
pub trait Attribute: Debug + 'static {
    /// The name of the attribute.
    fn name(&self) -> &str;

    /// Write the attribute to an output buffer.
    fn write(&self, pool: &mut PoolWriter, out: &mut Cursor<Vec<u8>>) -> Result<(), Error>;
}

/// A table of attributes that could be added to a class file.
#[derive(Debug, Default)]
pub struct AttributeTable(Vec<Box<dyn Attribute>>);
impl AttributeTable {
    pub fn push(&mut self, attr: impl Attribute) {
        self.0.push(Box::new(attr));
    }

    pub fn write(&self, pool: &mut PoolWriter, mut out: impl Write) -> Result<(), Error> {
        assert!(self.0.len() <= u16::MAX as usize);
        out.write_u16::<BE>(self.0.len() as u16)?;
        for attr in &self.0 {
            pool.utf8(attr.name()).write(&mut out)?;
            let mut cursor = Cursor::new(Vec::<u8>::new());
            attr.write(pool, &mut cursor)?;
            let cursor = cursor.into_inner();
            assert!(cursor.len() <= u32::MAX as usize);
            out.write_u32::<BE>(cursor.len() as u32)?;
            out.write_all(&cursor)?;
        }
        Ok(())
    }
}

/// Represents the `SourceFile` attribute of a Java class.
#[derive(Debug)]
pub struct SourceFile(String);
impl SourceFile {
    pub fn new(name: &str) -> Self {
        SourceFile(name.to_string())
    }
}
impl Attribute for SourceFile {
    fn name(&self) -> &str {
        "SourceFile"
    }
    fn write(&self, pool: &mut PoolWriter, out: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        pool.utf8(&self.0).write(out)
    }
}
