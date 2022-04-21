use crate::{
    classfile::{
        utils::{push_param, return_param},
        ClassWriter, MethodWriter,
    },
    signatures::{BasicType, ClassName, MethodSig, Type},
    CFlags, ClassData, FFlags, MFlags,
};
use enumset::EnumSet;
use std::collections::HashMap;

pub struct NativeClassWrapper {
    name: String,
    class: ClassWriter,
    extends: String,
    id_param: String,
    supporting: HashMap<String, Vec<u8>>,
    constructor_generated: bool,
}
impl NativeClassWrapper {
    pub fn new(access: EnumSet<CFlags>, name: &str, extends: &str, id_param: &str) -> Self {
        let mut class = ClassWriter::new(access, name);

        class.extends(extends);
        class.field(FFlags::Private | FFlags::Synthetic | FFlags::Final, id_param, "I");

        NativeClassWrapper {
            name: name.to_string(),
            class,
            extends: extends.to_string(),
            id_param: id_param.to_string(),
            supporting: HashMap::new(),
            constructor_generated: false,
        }
    }
    pub fn implements(&mut self, implement: &str) {
        self.class.implements(implement);
    }
    pub fn source_file(&mut self, source_file: &str) {
        self.class.source_file(source_file);
    }

    pub fn generate_init(&mut self, init_class: &str) {
        let method = self.class.method(MFlags::Static.into(), "<clinit>", "()V");
        let mut code = method.code();
        code.invokestatic(init_class, "init", "()V").vreturn();
    }

    pub fn export_constructor(
        &mut self,
        access: EnumSet<MFlags>,
        sig_str: &str,
        native_name: &str,
        native_sig_str: &str,
        super_sig_str: &str,
        late_init: &[&'static str],
    ) {
        self.constructor_generated = true;

        // parse the signatures passed in
        let sig = MethodSig::parse_jni(sig_str).unwrap();
        let native_sig = MethodSig::parse_jni(native_sig_str).unwrap();
        let super_sig = MethodSig::parse_jni(super_sig_str).unwrap();

        // parse out the intermediate type.
        let intermediate_name = match &native_sig.ret_ty {
            Type { basic_sig: BasicType::Int, .. } => None,
            Type { basic_sig: BasicType::Class(name), .. } => {
                let name_parsed = ClassName::parse_jni(&self.name).unwrap();
                assert_eq!(name_parsed.package, name.package);
                Some(name.display_jni().to_string())
            }
            _ => panic!("illegal ret_ty for native_sig"),
        };

        // check parameter consistancy
        assert_eq!(sig.ret_ty, Type::Void);
        assert_eq!(super_sig.ret_ty, Type::Void);
        assert_eq!(sig.params, native_sig.params);

        let method = self.class.method(access, "<init>", sig_str);
        let mut code = method.code();

        // call the actual native initialization function
        let mut param_id = 1;
        for param in sig.params.as_slice() {
            param_id += push_param(&mut code, param_id, param);
        }
        let var_native_ret = param_id;
        code.invokestatic(&self.name, native_name, native_sig_str);

        // write the actual Java-side initialization
        match intermediate_name {
            None => {
                // just call the superclass' default constructor
                code.aload(0)
                    .invokestatic(&self.extends, "<init>", "()V")
                    .putfield(&self.name, &self.id_param, "I")
                    .vreturn();
            }
            Some(support_name) => {
                // generate a supporting class
                {
                    let mut supporting =
                        ClassWriter::new(CFlags::Final | CFlags::Synthetic, &support_name);

                    // creating a constructor
                    {
                        // create the parameters of the constructor
                        let mut vec = Vec::new();
                        vec.push(Type::Int);
                        for param in super_sig.params.as_slice() {
                            vec.push(param.clone());
                        }

                        let mut method =
                            supporting.method(MFlags::Synthetic.into(), "<init>", "()V");
                        let mut code = method.code();

                        // write the id field
                        code.aload(0)
                            .invokestatic("java/lang/Object", "<init>", "()V")
                            .iload(1)
                            .putfield(&support_name, "id", "I");

                        // write the rest of the parameters
                        let mut id = 0;
                        let mut param_id = 2;
                        for param in sig.params.as_slice() {
                            let field_name = format!("param_{}", param_id);
                            let param_ty = param.display_jni().to_string();
                            param_id += push_param(&mut code, param_id, param);
                            code.putfield(&support_name, &field_name, &param_ty);
                            id += 1;
                        }

                        code.vreturn();
                    }

                    // write the id field
                    supporting.field(FFlags::Final | FFlags::Synthetic, "id", "I");

                    // write the rest of the parameters in the supporting class
                    let mut id = 0;
                    for param in sig.params.as_slice() {
                        let field_name = format!("param_{}", param_id);
                        let param_ty = param.display_jni().to_string();
                        supporting.field(FFlags::Final | FFlags::Synthetic, &field_name, &param_ty);
                        id += 1;
                    }

                    // write the class data
                    self.supporting
                        .insert(support_name.clone(), supporting.into_vec());
                }

                // load the parameters from the support class and call the super constructor
                code.astore(var_native_ret).aload(0);
                let mut id = 0;
                for param in native_sig.params.as_slice() {
                    let field_name = format!("param_{}", param_id);
                    let param_ty = param.display_jni().to_string();
                    code.aload(var_native_ret)
                        .getfield(&support_name, &field_name, &param_ty);
                    id += 1;
                }
                code.invokespecial(&self.extends, "<init>", native_sig_str)
                    .aload(0)
                    .aload(var_native_ret)
                    .getfield(&support_name, "id", "I")
                    .putfield(&self.name, "id", "I");
            }
        }

        // call all late init functions
        for func in late_init {
            code.aload(0).invokevirtual(&self.name, func, "()V");
        }

        // cleanup
        code.vreturn();
    }

    pub fn export_field(&mut self, access: EnumSet<FFlags>, name: &str, ty: &str) {
        self.class.field(access, name, ty);
    }

    pub fn export_native(&mut self, name: &str, sig: &str, is_static: bool) {
        let mut access = MFlags::Private | MFlags::Synthetic | MFlags::Native;
        if is_static {
            access |= MFlags::Static;
        }
        self.class.method(access, name, sig);
    }

    pub fn export_native_wrapper(
        &mut self,
        access: EnumSet<MFlags>,
        name: &str,
        sig_str: &str,
        native_name: &str,
        native_sig_str: &str,
        has_id_param: bool,
    ) {
        // parse the signatures passed in
        let sig = MethodSig::parse_jni(sig_str).unwrap();
        let native_sig = MethodSig::parse_jni(native_sig_str).unwrap();

        // begin generating the method
        let mut method = self.class.method(access, name, sig_str);
        let mut code = method.code();

        // retrieve parameters
        let native_sig_params = if has_id_param {
            &native_sig.params[1..]
        } else {
            &native_sig.params
        };

        // validate parameters
        if has_id_param {
            assert_eq!(native_sig.params[0], Type::Int);
        }
        assert_eq!(sig.ret_ty, native_sig.ret_ty);
        assert_eq!(native_sig_params, sig.params.as_slice());

        // generate trampoline to call the native method underlying this
        let mut param_id = if !access.contains(MFlags::Static) {
            code.aload(0);
            1
        } else {
            0
        };
        if has_id_param {
            code.dup().getfield(&self.name, &self.id_param, "I");
        }
        for param in native_sig_params {
            param_id += push_param(&mut code, param_id, &param);
        }
        if access.contains(MFlags::Static) {
            code.invokestatic(&self.name, native_name, native_sig_str);
        } else {
            code.invokevirtual(&self.name, native_name, native_sig_str);
        }
        return_param(&mut code, &sig.ret_ty);
    }

    pub(crate) fn add_to_jar(mut self, data: &mut ClassData) {
        // generate an empty constructor if there are none
        if !self.constructor_generated {
            let method = self.class.method(MFlags::Public.into(), "<init>", "()V");
            let mut code = method.code();
            code.aload(0)
                .invokespecial(&self.extends, "<init>", "()V")
                .vreturn();
        }

        // generate code
        data.add_class(&self.name, self.class.into_vec());
        for (name, class_data) in self.supporting {
            data.add_class(&name, class_data);
        }
    }
}
