mod attributes;
mod code;
mod constant_pool;
pub mod utils;

pub use code::MethodWriter;

use crate::{
    classfile::{
        attributes::{AttributeTable, SourceFile},
        constant_pool::{PoolId, PoolWriter},
    },
    signatures::MethodSig,
    CFlags, FFlags, MFlags,
};
use byteorder::{WriteBytesExt, BE};
use enumset::EnumSet;
use std::{
    io::{Cursor, Error, Write},
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct FieldData {
    access: EnumSet<FFlags>,
    name: String,
    jni_sig: String,
    attributes: AttributeTable,
}

#[derive(Debug)]
pub struct MethodData {
    access: EnumSet<MFlags>,
    name: String,
    jni_sig: String,
    attributes: AttributeTable,

    code_written: bool,
    arg_stack_size: usize,
}
impl MethodData {
    pub fn code(&mut self) -> MethodWriterGuard {
        assert!(!self.code_written);
        let mut writer = MethodWriter::default();
        writer.argc(self.arg_stack_size);
        MethodWriterGuard { data: self, writer }
    }
}

pub struct MethodWriterGuard<'a> {
    data: &'a mut MethodData,
    writer: MethodWriter,
}
impl<'a> Deref for MethodWriterGuard<'a> {
    type Target = MethodWriter;
    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}
impl<'a> DerefMut for MethodWriterGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}
impl<'a> Drop for MethodWriterGuard<'a> {
    fn drop(&mut self) {
        self.data.code_written = true;
        self.data
            .attributes
            .push(std::mem::replace(&mut self.writer, MethodWriter::default()));
    }
}

#[derive(Debug)]
pub struct ClassWriter {
    pool: PoolWriter,

    access_flags: EnumSet<CFlags>,
    this_class: PoolId,
    extends: Option<PoolId>,
    implements: Vec<PoolId>,

    fields: Vec<FieldData>,
    methods: Vec<MethodData>,
    attributes: AttributeTable,

    source_file_written: bool,
}
impl ClassWriter {
    pub fn new(access_flags: EnumSet<CFlags>, name: &str) -> Self {
        let mut pool = PoolWriter::default();
        let name = pool.class(name);
        ClassWriter {
            pool,
            access_flags,
            this_class: name,
            extends: None,
            implements: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Default::default(),
            source_file_written: false,
        }
    }

    pub fn extends(&mut self, name: &str) -> &mut Self {
        assert!(self.extends.is_none());
        self.extends = Some(self.pool.class(name));
        self
    }
    pub fn implements(&mut self, name: &str) -> &mut Self {
        self.implements.push(self.pool.class(name));
        self
    }

    pub fn field(&mut self, access: EnumSet<FFlags>, name: &str, ty: &str) -> &mut FieldData {
        let field = FieldData {
            access,
            name: name.to_string(),
            jni_sig: ty.to_string(),
            attributes: Default::default(),
        };
        self.fields.push(field);
        self.fields.last_mut().unwrap()
    }
    pub fn method(&mut self, access: EnumSet<MFlags>, name: &str, ty: &str) -> &mut MethodData {
        let method = MethodData {
            access,
            name: name.to_string(),
            jni_sig: ty.to_string(),
            attributes: Default::default(),
            code_written: false,
            arg_stack_size: {
                let mut count = if access.contains(MFlags::Static) { 0 } else { 1 };
                for param in MethodSig::parse_jni(ty).unwrap().params {
                    count += utils::type_stack_size(&param);
                }
                count as usize
            },
        };
        self.methods.push(method);
        self.methods.last_mut().unwrap()
    }

    pub fn source_file(&mut self, file: &str) -> &mut Self {
        assert!(!self.source_file_written);
        self.source_file_written = true;
        self.attributes.push(SourceFile::new(file));
        self
    }

    pub fn write(mut self, mut write: impl Write) -> Result<(), Error> {
        write.write_u32::<BE>(0xCAFEBABE)?;

        // classfile version 52.0 (Java 8)
        write.write_u16::<BE>(0)?;
        write.write_u16::<BE>(52)?;

        // write classfile body
        let mut body = Cursor::new(Vec::<u8>::new());
        body.write_u16::<BE>(self.access_flags.as_u16())?;
        self.this_class.write(&mut body)?;
        if let Some(extends) = self.extends {
            extends.write(&mut body)?;
        } else {
            self.pool.class("java/lang/Object").write(&mut body)?;
        }
        assert!(self.implements.len() <= u16::MAX as usize);
        body.write_u16::<BE>(self.implements.len() as u16)?;
        for interface in &self.implements {
            interface.write(&mut body)?;
        }

        // write fields
        assert!(self.fields.len() <= u16::MAX as usize);
        body.write_u16::<BE>(self.fields.len() as u16)?;
        for field in &self.fields {
            body.write_u16::<BE>(field.access.as_u16())?;
            self.pool.utf8(field.name.as_str()).write(&mut body)?;
            self.pool.utf8(field.jni_sig.as_str()).write(&mut body)?;
            field.attributes.write(&mut self.pool, &mut body)?;
        }

        // write methods
        assert!(self.methods.len() <= u16::MAX as usize);
        body.write_u16::<BE>(self.methods.len() as u16)?;
        for method in &self.methods {
            body.write_u16::<BE>(method.access.as_u16())?;
            self.pool.utf8(method.name.as_str()).write(&mut body)?;
            self.pool.utf8(method.jni_sig.as_str()).write(&mut body)?;
            method.attributes.write(&mut self.pool, &mut body)?;
        }

        // write attributes
        self.attributes.write(&mut self.pool, &mut body)?;

        // write constant pool (this is the last step because the constant pool contains so much.)
        assert!(self.pool.len() <= u16::MAX as usize);
        write.write_u16::<BE>(self.pool.len() as u16 + 1)?;
        write.write_all(self.pool.contents())?;

        // write body to the classfile
        write.write_all(body.get_ref().as_slice())?;

        Ok(())
    }
    pub fn into_vec(self) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        self.write(&mut cursor)
            .expect("Could not generate Java classfile.");
        cursor.into_inner()
    }
}
