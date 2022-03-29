use byteorder::{WriteBytesExt, BE};
use nekojni_signatures::{ClassName, MethodSig, Type};
use std::{
    collections::HashMap,
    io::{Cursor, Error, Write},
};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PoolId(u16);
impl PoolId {
    pub fn write(&self, mut w: impl Write) -> Result<(), Error> {
        w.write_u16::<BE>(self.0)
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum PoolEntry {
    Utf8(String),
    Integer(i32),
    Float(u32), // f32 bits
    Long(i64),
    Double(u64), // f64 bits
    Class(PoolId),
    String(PoolId),
    FieldRef {
        class_index: PoolId,
        name_and_type_index: PoolId,
    },
    MethodRef {
        class_index: PoolId,
        name_and_type_index: PoolId,
    },
    InterfaceMethodRef {
        class_index: PoolId,
        name_and_type_index: PoolId,
    },
    NameAndType {
        name_index: PoolId,
        descriptor_index: PoolId,
    },
}

#[derive(Default, Debug)]
pub struct PoolWriter {
    writer: Cursor<Vec<u8>>,
    cache: HashMap<PoolEntry, PoolId>,
}
impl PoolWriter {
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    pub fn contents(&self) -> &[u8] {
        self.writer.get_ref().as_slice()
    }

    fn write_entry(&mut self, entry: &PoolEntry) -> Result<(), Error> {
        match &entry {
            PoolEntry::Utf8(str) => {
                let cesu = cesu8::to_java_cesu8(str);
                assert!(cesu.len() <= u16::MAX as usize);

                self.writer.write_u8(1)?;
                self.writer.write_u16::<BE>(cesu.len() as u16)?;
                self.writer.write_all(&cesu)?;
            }
            PoolEntry::Integer(v) => {
                self.writer.write_u8(3)?;
                self.writer.write_i32::<BE>(*v)?;
            }
            PoolEntry::Float(v) => {
                self.writer.write_u8(4)?;
                self.writer.write_f32::<BE>(f32::from_bits(*v))?;
            }
            PoolEntry::Long(v) => {
                self.writer.write_u8(5)?;
                self.writer.write_i64::<BE>(*v)?;
            }
            PoolEntry::Double(v) => {
                self.writer.write_u8(6)?;
                self.writer.write_f64::<BE>(f64::from_bits(*v))?;
            }
            PoolEntry::Class(id) => {
                self.writer.write_u8(7)?;
                self.writer.write_u16::<BE>(id.0)?;
            }
            PoolEntry::String(str) => {
                self.writer.write_u8(8)?;
                self.writer.write_u16::<BE>(str.0)?;
            }
            PoolEntry::FieldRef {
                class_index,
                name_and_type_index,
            } => {
                self.writer.write_u8(9)?;
                self.writer.write_u16::<BE>(class_index.0)?;
                self.writer.write_u16::<BE>(name_and_type_index.0)?;
            }
            PoolEntry::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                self.writer.write_u8(10)?;
                self.writer.write_u16::<BE>(class_index.0)?;
                self.writer.write_u16::<BE>(name_and_type_index.0)?;
            }
            PoolEntry::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => {
                self.writer.write_u8(11)?;
                self.writer.write_u16::<BE>(class_index.0)?;
                self.writer.write_u16::<BE>(name_and_type_index.0)?;
            }
            PoolEntry::NameAndType {
                name_index,
                descriptor_index,
            } => {
                self.writer.write_u8(12)?;
                self.writer.write_u16::<BE>(name_index.0)?;
                self.writer.write_u16::<BE>(descriptor_index.0)?;
            }
        }
        Ok(())
    }
    fn entry(&mut self, entry: PoolEntry) -> PoolId {
        if let Some(x) = self.cache.get(&entry) {
            *x
        } else {
            let raw_id = self.cache.len() + 1;
            assert!(raw_id <= u16::MAX as usize);

            self.write_entry(&entry)
                .expect("Could not successfully write pool entry?");

            let id = PoolId(raw_id as u16);
            self.cache.insert(entry, id);
            id
        }
    }
    fn name_and_type(&mut self, v: &str, descriptor: &str) -> PoolId {
        let name_index = self.utf8(v);
        let descriptor_index = self.utf8(descriptor);
        self.entry(PoolEntry::NameAndType {
            name_index,
            descriptor_index,
        })
    }

    pub fn utf8(&mut self, v: &str) -> PoolId {
        self.entry(PoolEntry::Utf8(v.to_string()))
    }
    pub fn integer(&mut self, v: i32) -> PoolId {
        self.entry(PoolEntry::Integer(v))
    }
    pub fn float(&mut self, v: f32) -> PoolId {
        self.entry(PoolEntry::Float(v.to_bits()))
    }
    pub fn long(&mut self, v: i64) -> PoolId {
        self.entry(PoolEntry::Long(v))
    }
    pub fn double(&mut self, v: f64) -> PoolId {
        self.entry(PoolEntry::Double(v.to_bits()))
    }
    pub fn class(&mut self, v: &ClassName) -> PoolId {
        let contents = self.utf8(&v.display_jni().to_string());
        self.entry(PoolEntry::Class(contents))
    }
    pub fn class_str(&mut self, v: &str) -> PoolId {
        let contents = self.utf8(&v);
        self.entry(PoolEntry::Class(contents))
    }
    pub fn string(&mut self, str: &str) -> PoolId {
        let contents = self.utf8(str);
        self.entry(PoolEntry::String(contents))
    }
    pub fn field_ref_str(&mut self, cl: &str, name: &str, ty: &str) -> PoolId {
        let class = self.class_str(cl);
        let name_and_type = self.name_and_type(name, &ty);
        self.entry(PoolEntry::FieldRef {
            class_index: class,
            name_and_type_index: name_and_type,
        })
    }
    pub fn method_ref_str(&mut self, cl: &str, name: &str, ty: &str) -> PoolId {
        let class = self.class_str(cl);
        let name_and_type = self.name_and_type(name, &ty);
        self.entry(PoolEntry::MethodRef {
            class_index: class,
            name_and_type_index: name_and_type,
        })
    }
    pub fn interface_method_ref_str(&mut self, cl: &str, name: &str, ty: &str) -> PoolId {
        let class = self.class_str(cl);
        let name_and_type = self.name_and_type(name, &ty);
        self.entry(PoolEntry::InterfaceMethodRef {
            class_index: class,
            name_and_type_index: name_and_type,
        })
    }
}
