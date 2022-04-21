#![allow(unused)]

use crate::{
    classfile::{attributes::Attribute, PoolId, PoolWriter},
    signatures::{MethodSig, Type},
};
use byteorder::{WriteBytesExt, BE};
use std::{
    collections::HashMap,
    io::{Cursor, Error, Write},
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Default)]
pub struct MethodWriter {
    cur_stack: isize,
    max_stack: isize,
    max_field: u16,
    instr: Vec<Instruction>,
}
impl Attribute for MethodWriter {
    fn name(&self) -> &str {
        "Code"
    }
    fn write(&self, pool: &mut PoolWriter, out: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        // write header
        out.write_u16::<BE>(self.max_stack as u16)?;
        out.write_u16::<BE>(self.max_field)?;

        // write code
        let mut code = Cursor::new(Vec::<u8>::new());
        for instr in &self.instr {
            match instr {
                _ => instr.write(pool, &mut code)?,
            }
        }
        let code = code.into_inner();
        assert!(code.len() <= u32::MAX as usize);
        out.write_u32::<BE>(code.len() as u32)?;
        out.write_all(&code)?;

        // TODO: exception table
        out.write_u16::<BE>(0)?;
        // TODO: attributes table
        out.write_u16::<BE>(0)?;

        Ok(())
    }
}
impl MethodWriter {
    #[inline(never)]
    fn push_instr(&mut self, instr: Instruction) -> &mut Self {
        // adjust the stack size based on this instruction.
        self.cur_stack += instr.stack_change();
        assert!(self.cur_stack <= i16::MAX as isize);
        assert!(self.cur_stack >= 0);
        self.max_stack = self.max_stack.max(self.cur_stack);

        // push the actual instruction itself
        self.instr.push(instr);
        self
    }

    pub(crate) fn argc(&mut self, count: usize) -> &mut Self {
        assert!(count <= u16::MAX as usize);
        self.max_field = self.max_field.max(count as u16);
        self
    }

    #[inline(never)]
    pub fn invokeinterface(&mut self, class: &str, name: &str, sig: &str) -> &mut Self {
        self.push_instr(Instruction::invokeinterface(InvokeData::new(class, name, sig)))
    }
    #[inline(never)]
    pub fn invokespecial(&mut self, class: &str, name: &str, sig: &str) -> &mut Self {
        self.push_instr(Instruction::invokespecial(InvokeData::new(class, name, sig)))
    }
    #[inline(never)]
    pub fn invokestatic(&mut self, class: &str, name: &str, sig: &str) -> &mut Self {
        self.push_instr(Instruction::invokestatic(InvokeData::new(class, name, sig)))
    }
    #[inline(never)]
    pub fn invokevirtual(&mut self, class: &str, name: &str, sig: &str) -> &mut Self {
        self.push_instr(Instruction::invokevirtual(InvokeData::new(class, name, sig)))
    }

    #[inline(never)]
    pub fn new(&mut self, ty: &str) -> &mut Self {
        self.push_instr(Instruction::new(ty.to_string()))
    }
    #[inline(never)]
    pub fn anewarray(&mut self, ty: &str) -> &mut Self {
        self.push_instr(Instruction::anewarray(ty.to_string()))
    }
    #[inline(never)]
    pub fn checkcast(&mut self, ty: &str) -> &mut Self {
        self.push_instr(Instruction::checkcast(ty.to_string()))
    }

    #[inline(never)]
    pub fn getfield(&mut self, class: &str, name: &str, ty: &str) -> &mut Self {
        self.push_instr(Instruction::getfield(FieldData::new(class, name, ty)))
    }
    #[inline(never)]
    pub fn getstatic(&mut self, class: &str, name: &str, ty: &str) -> &mut Self {
        self.push_instr(Instruction::getstatic(FieldData::new(class, name, ty)))
    }
    #[inline(never)]
    pub fn putfield(&mut self, class: &str, name: &str, ty: &str) -> &mut Self {
        self.push_instr(Instruction::putfield(FieldData::new(class, name, ty)))
    }
    #[inline(never)]
    pub fn putstatic(&mut self, class: &str, name: &str, ty: &str) -> &mut Self {
        self.push_instr(Instruction::putstatic(FieldData::new(class, name, ty)))
    }

    #[inline(never)]
    pub fn aconst_str(&mut self, str: &str) -> &mut Self {
        self.push_instr(Instruction::aconst_str(str.to_string()))
    }
    #[inline(never)]
    pub fn aconst_class(&mut self, class: &str) -> &mut Self {
        self.push_instr(Instruction::aconst_class(class.to_string()))
    }
    #[inline(never)]
    pub fn fconst(&mut self, v: f32) -> &mut Self {
        if v == 0.0 {
            self.push_instr(Instruction::Basic(BasicInstruction::fconst_0))
        } else if v == 1.0 {
            self.push_instr(Instruction::Basic(BasicInstruction::fconst_1))
        } else if v == 2.0 {
            self.push_instr(Instruction::Basic(BasicInstruction::fconst_2))
        } else {
            self.push_instr(Instruction::fconst(v))
        }
    }
    #[inline(never)]
    pub fn iconst(&mut self, v: i32) -> &mut Self {
        match v {
            -1 => self.push_instr(Instruction::Basic(BasicInstruction::iconst_m1)),
            0 => self.push_instr(Instruction::Basic(BasicInstruction::iconst_0)),
            1 => self.push_instr(Instruction::Basic(BasicInstruction::iconst_1)),
            2 => self.push_instr(Instruction::Basic(BasicInstruction::iconst_2)),
            3 => self.push_instr(Instruction::Basic(BasicInstruction::iconst_3)),
            4 => self.push_instr(Instruction::Basic(BasicInstruction::iconst_4)),
            5 => self.push_instr(Instruction::Basic(BasicInstruction::iconst_5)),
            _ => self.push_instr(Instruction::iconst(v)),
        }
    }
    #[inline(never)]
    pub fn dconst(&mut self, v: f64) -> &mut Self {
        if v == 0.0 {
            self.push_instr(Instruction::Basic(BasicInstruction::dconst_0))
        } else if v == 1.0 {
            self.push_instr(Instruction::Basic(BasicInstruction::dconst_1))
        } else {
            self.push_instr(Instruction::dconst(v))
        }
    }
    #[inline(never)]
    pub fn lconst(&mut self, v: i64) -> &mut Self {
        match v {
            0 => self.push_instr(Instruction::Basic(BasicInstruction::lconst_0)),
            1 => self.push_instr(Instruction::Basic(BasicInstruction::lconst_1)),
            _ => self.push_instr(Instruction::lconst(v)),
        }
    }
}

#[derive(Debug)]
struct FieldData {
    class: String,
    name: String,
    desc: String,
    slots: usize,
}
impl FieldData {
    fn new(class: &str, name: &str, ty: &str) -> Self {
        let parsed_ty = Type::parse_jni(ty).unwrap();
        FieldData {
            class: class.to_string(),
            name: name.to_string(),
            desc: ty.to_string(),
            slots: if parsed_ty == Type::Double || parsed_ty == Type::Long {
                2
            } else {
                1
            },
        }
    }
    fn make_ref(&self, pool: &mut PoolWriter) -> PoolId {
        pool.field_ref(&self.class, &self.name, &self.desc)
    }
}

#[derive(Debug)]
struct InvokeData {
    class: String,
    name: String,
    desc: String,
    argc: usize,
    retc: usize,
}
impl InvokeData {
    fn new(class: &str, name: &str, sig: &str) -> Self {
        let parsed_sig = MethodSig::parse_jni(sig).unwrap();
        InvokeData {
            class: class.to_string(),
            name: name.to_string(),
            desc: sig.to_string(),
            argc: parsed_sig.params.len(),
            retc: if parsed_sig.ret_ty == Type::Void { 0 } else { 1 },
        }
    }
    fn make_method_ref(&self, pool: &mut PoolWriter) -> PoolId {
        pool.method_ref(&self.class, &self.name, &self.desc)
    }
    fn make_interface_method_ref(&self, pool: &mut PoolWriter) -> PoolId {
        pool.interface_method_ref(&self.class, &self.name, &self.desc)
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum Instruction {
    invokeinterface(InvokeData),
    invokespecial(InvokeData),
    invokestatic(InvokeData),
    invokevirtual(InvokeData),

    new(String),
    anewarray(String),
    checkcast(String),
    getfield(FieldData),
    getstatic(FieldData),
    putfield(FieldData),
    putstatic(FieldData),

    aconst_str(String),
    aconst_class(String),
    fconst(f32),
    iconst(i32),
    dconst(f64),
    lconst(i64),

    aload(u16),
    fload(u16),
    iload(u16),
    dload(u16),
    lload(u16),
    astore(u16),
    fstore(u16),
    istore(u16),
    dstore(u16),
    lstore(u16),

    Basic(BasicInstruction),
}
impl Instruction {
    //noinspection SpellCheckingInspection
    fn stack_change(&self) -> isize {
        use BasicInstruction::*;
        use Instruction::*;

        match self {
            invokeinterface(data) => -(data.argc as isize + 1) + data.retc as isize,
            invokespecial(data) => -(data.argc as isize + 1) + data.retc as isize,
            invokestatic(data) => -(data.argc as isize) + data.retc as isize,
            invokevirtual(data) => -(data.argc as isize + 1) + data.retc as isize,
            getfield(data) => data.slots as isize - 1,
            getstatic(data) => data.slots as isize,
            putfield(data) => -(data.slots as isize) - 1,
            putstatic(data) => -(data.slots as isize),

            new(_) => 1,
            anewarray(_) => 0,
            checkcast(_) => 0,

            aconst_class(_) | aconst_str(_) | fconst(_) | iconst(_) => 1,
            dconst(_) | lconst(_) => 2,

            aload(_) | fload(_) | iload(_) => 1,
            dload(_) | lload(_) => 2,
            astore(_) | fstore(_) | istore(_) => -1,
            dstore(_) | lstore(_) => -2,

            Basic(instr) => {
                match instr {
                    // array load
                    aaload | baload | caload | faload | iaload | saload => -1,
                    aastore | bastore | castore | fastore | iastore | sastore => -3,
                    daload | laload => 0,
                    dastore | lastore => -4,

                    // short-hand store
                    aload_0 | aload_1 | aload_2 | aload_3 => 1,
                    fload_0 | fload_1 | fload_2 | fload_3 => 1,
                    iload_0 | iload_1 | iload_2 | iload_3 => 1,
                    astore_0 | astore_1 | astore_2 | astore_3 => -1,
                    fstore_0 | fstore_1 | fstore_2 | fstore_3 => -1,
                    istore_0 | istore_1 | istore_2 | istore_3 => -1,
                    dload_0 | dload_1 | dload_2 | dload_3 => 2,
                    lload_0 | lload_1 | lload_2 | lload_3 => 2,
                    dstore_0 | dstore_1 | dstore_2 | dstore_3 => -2,
                    lstore_0 | lstore_1 | lstore_2 | lstore_3 => -2,

                    // constant values
                    aconst_null | fconst_0 | fconst_1 | fconst_2 | iconst_m1 | iconst_0
                    | iconst_1 | iconst_2 | iconst_3 | iconst_4 | iconst_5 => 1,
                    dconst_0 | dconst_1 | lconst_0 | lconst_1 => 2,

                    // conversions
                    d2f | d2i | l2f => -1,
                    d2l | f2i | i2b | i2c | i2f | i2s | l2d | l2i => 0,
                    f2d | f2l | i2d | i2l => 1,

                    // mathematical operations
                    dadd | dcmpg | dcmpl | ddiv | dmul | drem | dsub => -2,
                    dneg => 0,
                    fadd | fcmpg | fcmpl | fdiv | fmul | frem | fsub => -1,
                    fneg => 0,
                    iadd | idiv | imul | irem | isub => -1,
                    iand | ior | ishl | ishr | iushr | ixor => -1,
                    ineg => 0,
                    ladd | ldiv | lmul | lrem | lsub => -2,
                    land | lor | lshl | lshr | lushr | lxor => -2,
                    lcmp => -3,
                    lneg => 0,

                    // stack manipulation command
                    dup | dup_x1 | dup_x2 => 1,
                    dup2 | dup2_x1 | dup2_x2 => 2,
                    pop => -1,
                    pop2 => -2,
                    swap => 0,

                    // misc commands
                    vreturn => 0,
                    areturn | freturn | ireturn => -1,
                    dreturn | lreturn => -2,
                    breakpoint | nop => 0,
                    monitorenter | monitorexit => -1,
                    arraylength => 0,
                    athrow => -1,
                }
            }
        }
    }

    fn write_id(&self, mut out: impl Write, opcode: u8, offset: u16) -> Result<(), Error> {
        if offset <= 255 {
            out.write_u8(opcode)?;
            out.write_u8(offset as u8)?;
        } else {
            out.write_u8(0xc4)?; // wide prefix
            out.write_u8(opcode)?;
            out.write_u16::<BE>(offset)?;
        }
        Ok(())
    }
    fn write(&self, pool: &mut PoolWriter, mut out: impl Write) -> Result<(), Error> {
        match self {
            Instruction::invokeinterface(data) => {
                out.write_u8(0xb9)?;
                data.make_interface_method_ref(pool).write(&mut out)?;
                out.write_u8((data.retc + 1) as u8)?;
                out.write_u8(0)?;
            }
            Instruction::invokespecial(data) => {
                out.write_u8(0xb7)?;
                data.make_method_ref(pool).write(&mut out)?;
            }
            Instruction::invokestatic(data) => {
                out.write_u8(0xb8)?;
                data.make_method_ref(pool).write(&mut out)?;
            }
            Instruction::invokevirtual(data) => {
                out.write_u8(0xb6)?;
                data.make_method_ref(pool).write(&mut out)?;
            }
            Instruction::getfield(data) => {
                out.write_u8(0xb4)?;
                data.make_ref(pool).write(&mut out)?;
            }
            Instruction::getstatic(data) => {
                out.write_u8(0xb2)?;
                data.make_ref(pool).write(&mut out)?;
            }
            Instruction::putfield(data) => {
                out.write_u8(0xb5)?;
                data.make_ref(pool).write(&mut out)?;
            }
            Instruction::putstatic(data) => {
                out.write_u8(0xb3)?;
                data.make_ref(pool).write(&mut out)?;
            }
            Instruction::new(ty) => {
                out.write_u8(0xbb)?;
                pool.class(ty).write(&mut out)?;
            }
            Instruction::anewarray(ty) => {
                out.write_u8(0xbd)?;
                pool.class(ty).write(&mut out)?;
            }
            Instruction::checkcast(ty) => {
                out.write_u8(0xc0)?;
                pool.class(ty).write(&mut out)?;
            }
            Instruction::aconst_class(str) => {
                out.write_u8(0x13)?; // ldc_w
                pool.class(str).write(&mut out)?;
            }
            Instruction::aconst_str(str) => {
                out.write_u8(0x13)?; // ldc_w
                pool.string(str).write(&mut out)?;
            }
            Instruction::fconst(v) => {
                out.write_u8(0x13)?; // ldc_w
                pool.float(*v).write(&mut out)?;
            }
            Instruction::iconst(v) => {
                if *v <= i8::MAX as i32 && *v >= i8::MIN as i32 {
                    out.write_u8(0x10)?; // bipush
                    out.write_i8(*v as i8)?;
                } else {
                    out.write_u8(0x13)?; // ldc
                    pool.integer(*v).write(&mut out)?;
                }
            }
            Instruction::dconst(v) => {
                out.write_u8(0x14)?; // ldc2_w
                pool.double(*v).write(&mut out)?;
            }
            Instruction::lconst(v) => {
                out.write_u8(0x14)?; // ldc2_w
                pool.long(*v).write(&mut out)?;
            }
            Instruction::aload(id) => self.write_id(&mut out, 0x19, *id)?,
            Instruction::fload(id) => self.write_id(&mut out, 0x17, *id)?,
            Instruction::iload(id) => self.write_id(&mut out, 0x15, *id)?,
            Instruction::dload(id) => self.write_id(&mut out, 0x18, *id)?,
            Instruction::lload(id) => self.write_id(&mut out, 0x16, *id)?,
            Instruction::astore(id) => self.write_id(&mut out, 0x3a, *id)?,
            Instruction::fstore(id) => self.write_id(&mut out, 0x38, *id)?,
            Instruction::istore(id) => self.write_id(&mut out, 0x36, *id)?,
            Instruction::dstore(id) => self.write_id(&mut out, 0x39, *id)?,
            Instruction::lstore(id) => self.write_id(&mut out, 0x37, *id)?,
            Instruction::Basic(instr) => {
                out.write_u8(instr.opcode())?;
            }
        }
        Ok(())
    }
}

macro_rules! basic_instruction {
    ($($hex:literal $name:ident,)*) => {
        #[derive(Copy, Clone, Debug)]
        #[allow(non_camel_case_types)]
        enum BasicInstruction {
            $($name,)*
        }
        impl BasicInstruction {
            fn opcode(&self) -> u8 {
                match *self {
                    $(BasicInstruction::$name => $hex,)*
                }
            }
        }
    };
}
basic_instruction! {
    0x32 aaload,
    0x53 aastore,
    0x01 aconst_null,
    0x2a aload_0,
    0x2b aload_1,
    0x2c aload_2,
    0x2d aload_3,
    0xb0 areturn,
    0xbe arraylength,
    0x4b astore_0,
    0x4c astore_1,
    0x4d astore_2,
    0x4e astore_3,
    0xbf athrow,
    0x33 baload,
    0x54 bastore,
    0xca breakpoint,
    0x34 caload,
    0x55 castore,
    0x90 d2f,
    0x8e d2i,
    0x8f d2l,
    0x63 dadd,
    0x31 daload,
    0x52 dastore,
    0x98 dcmpg,
    0x97 dcmpl,
    0x0e dconst_0,
    0x0f dconst_1,
    0x6f ddiv,
    0x26 dload_0,
    0x27 dload_1,
    0x28 dload_2,
    0x29 dload_3,
    0x6b dmul,
    0x77 dneg,
    0x73 drem,
    0xaf dreturn,
    0x47 dstore_0,
    0x48 dstore_1,
    0x49 dstore_2,
    0x4a dstore_3,
    0x67 dsub,
    0x59 dup,
    0x5a dup_x1,
    0x5b dup_x2,
    0x5c dup2,
    0x5d dup2_x1,
    0x5e dup2_x2,
    0x8d f2d,
    0x8b f2i,
    0x8c f2l,
    0x62 fadd,
    0x30 faload,
    0x51 fastore,
    0x96 fcmpg,
    0x95 fcmpl,
    0x0b fconst_0,
    0x0c fconst_1,
    0x0d fconst_2,
    0x6e fdiv,
    0x22 fload_0,
    0x23 fload_1,
    0x24 fload_2,
    0x25 fload_3,
    0x6a fmul,
    0x76 fneg,
    0x72 frem,
    0xae freturn,
    0x43 fstore_0,
    0x44 fstore_1,
    0x45 fstore_2,
    0x46 fstore_3,
    0x66 fsub,
    0x91 i2b,
    0x92 i2c,
    0x87 i2d,
    0x86 i2f,
    0x85 i2l,
    0x93 i2s,
    0x60 iadd,
    0x2e iaload,
    0x7e iand,
    0x4f iastore,
    0x02 iconst_m1,
    0x03 iconst_0,
    0x04 iconst_1,
    0x05 iconst_2,
    0x06 iconst_3,
    0x07 iconst_4,
    0x08 iconst_5,
    0x6c idiv,
    0x1a iload_0,
    0x1b iload_1,
    0x1c iload_2,
    0x1d iload_3,
    0x68 imul,
    0x74 ineg,
    0x80 ior,
    0x70 irem,
    0xac ireturn,
    0x78 ishl,
    0x7a ishr,
    0x3b istore_0,
    0x3c istore_1,
    0x3d istore_2,
    0x3e istore_3,
    0x64 isub,
    0x7c iushr,
    0x82 ixor,
    0x8a l2d,
    0x89 l2f,
    0x88 l2i,
    0x61 ladd,
    0x2f laload,
    0x7f land,
    0x50 lastore,
    0x94 lcmp,
    0x09 lconst_0,
    0x0a lconst_1,
    0x6d ldiv,
    0x1e lload_0,
    0x1f lload_1,
    0x20 lload_2,
    0x21 lload_3,
    0x69 lmul,
    0x75 lneg,
    0x81 lor,
    0x71 lrem,
    0xad lreturn,
    0x79 lshl,
    0x7b lshr,
    0x3f lstore_0,
    0x40 lstore_1,
    0x41 lstore_2,
    0x42 lstore_3,
    0x65 lsub,
    0x7d lushr,
    0x83 lxor,
    0xc2 monitorenter,
    0xc3 monitorexit,
    0x00 nop,
    0x57 pop,
    0x58 pop2,
    0xb1 vreturn,
    0x35 saload,
    0x56 sastore,
    0x5f swap,
}

macro_rules! proxy_instructions {
    ($($instr:ident)*) => {
        impl MethodWriter {$(
            #[inline(never)]
            pub fn $instr(&mut self) -> &mut Self {
                self.push_instr(Instruction::Basic(BasicInstruction::$instr))
            }
        )*}
    };
}
proxy_instructions! {
    aaload aastore aconst_null areturn arraylength athrow baload bastore breakpoint caload castore
    d2f d2i d2l dadd daload dastore dcmpg dcmpl ddiv dmul dneg drem dreturn dsub dup dup_x1 dup_x2
    dup2 dup2_x1 dup2_x2 f2d f2i f2l fadd faload fastore fcmpg fcmpl fdiv fmul fneg frem freturn
    fsub i2b i2c i2d i2f i2l i2s iadd iaload iand iastore idiv imul ineg ior irem ireturn ishl ishr
    isub iushr ixor l2d l2f l2i ladd laload land lastore lcmp ldiv lmul lneg lor lrem lreturn lshl
    lshr lsub lushr lxor monitorenter monitorexit nop pop pop2 vreturn saload sastore swap
}

macro_rules! proxy_loadstore {
    (
        $(($l:ident $l0:ident $l1:ident $l2:ident $l3:ident, $ct:expr))*
    ) => {
        impl MethodWriter {$(
            #[inline(never)]
            pub fn $l(&mut self, id: u16) -> &mut Self {
                self.max_field = self.max_field.max(id + $ct);
                self.push_instr(match id {
                    0 => Instruction::Basic(BasicInstruction::$l0),
                    1 => Instruction::Basic(BasicInstruction::$l1),
                    2 => Instruction::Basic(BasicInstruction::$l2),
                    3 => Instruction::Basic(BasicInstruction::$l3),
                    _ => Instruction::$l(id),
                })
            }
        )*}
    };
}
proxy_loadstore! {
    (aload aload_0 aload_1 aload_2 aload_3, 1)
    (fload fload_0 fload_1 fload_2 fload_3, 1)
    (iload iload_0 iload_1 iload_2 iload_3, 1)
    (dload dload_0 dload_1 dload_2 dload_3, 2)
    (lload lload_0 lload_1 lload_2 lload_3, 2)
    (astore astore_0 astore_1 astore_2 astore_3, 1)
    (fstore fstore_0 fstore_1 fstore_2 fstore_3, 1)
    (istore istore_0 istore_1 istore_2 istore_3, 1)
    (dstore dstore_0 dstore_1 dstore_2 dstore_3, 2)
    (lstore lstore_0 lstore_1 lstore_2 lstore_3, 2)
}
