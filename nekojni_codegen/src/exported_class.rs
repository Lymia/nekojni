use crate::utils::{push_param, return_param, OBJECT_CL};
use enumset::EnumSet;
use nekojni_classfile::{CFlags, ClassWriter, FFlags, MFlags, MethodWriter};
use nekojni_signatures::{BasicType, ClassName, MethodSig, ReturnType, Type};
use std::collections::HashMap;

pub struct ClassExporter<'a> {
    name: &'a ClassName<'a>,
    class: ClassWriter,
    extends: &'a ClassName<'a>,
    id_param: &'a str,
    supporting: HashMap<String, Vec<u8>>,
}
impl<'a> ClassExporter<'a> {
    pub fn new(
        access: EnumSet<CFlags>,
        extends: &'a ClassName<'a>,
        name: &'a ClassName<'a>,
        id_param: &'a str,
    ) -> Self {
        let mut class = ClassWriter::new(access, name);

        class.extends(extends);
        class.field(FFlags::Private | FFlags::Synthetic | FFlags::Final, id_param, &Type::Int);

        ClassExporter { name, class, extends, id_param, supporting: HashMap::new() }
    }

    pub fn export_constructor(
        &mut self,
        access: EnumSet<MFlags>,
        sig: &MethodSig,
        native_name: &str,
        native_sig: &MethodSig,
        super_sig: &MethodSig,
        late_init: &[&'static str],
    ) {
        let intermediate_name = match &native_sig.ret_ty {
            ReturnType::Ty(Type { basic_sig: BasicType::Int, .. }) => None,
            ReturnType::Ty(Type { basic_sig: BasicType::Class(name), .. }) => {
                assert_eq!(self.name.package, name.package);
                Some(name)
            }
            _ => panic!("illegal ret_ty for native_sig"),
        };

        // check parameter consistancy
        assert_eq!(sig.ret_ty, ReturnType::Void);
        assert_eq!(super_sig.ret_ty, ReturnType::Void);
        assert_eq!(sig.params, native_sig.params);

        let method = self.class.method(access, "<init>", sig);
        let mut code = method.code();

        // call the actual native initialization function
        let mut param_id = 1;
        for param in sig.params.as_slice() {
            param_id += push_param(&mut code, param_id, param);
        }
        let var_native_ret = param_id;
        code.invokestatic(&self.name, native_name, native_sig);

        // write the actual Java-side initialization
        match intermediate_name {
            None => {
                // just call the superclass' default constructor
                code.aload(0)
                    .invokestatic(self.extends, "<init>", &MethodSig::void(&[]))
                    .putfield(&self.name, self.id_param, &Type::Int)
                    .vreturn();
            }
            Some(support_name) => {
                // generate a supporting class
                {
                    let mut supporting =
                        ClassWriter::new(CFlags::Final | CFlags::Synthetic, support_name);

                    // creating a constructor
                    {
                        // create the parameters of the constructor
                        let mut vec = Vec::new();
                        vec.push(Type::Int);
                        for param in super_sig.params.as_slice() {
                            vec.push(param.clone());
                        }

                        let mut method = supporting.method(
                            MFlags::Synthetic.into(),
                            "<init>",
                            &MethodSig::void(&vec),
                        );
                        let mut code = method.code();

                        // write the id field
                        code.aload(0)
                            .invokestatic(&OBJECT_CL, "<init>", &MethodSig::void(&[]))
                            .iload(1)
                            .putfield(support_name, "id", &Type::Int);

                        // write the rest of the parameters
                        let mut id = 0;
                        let mut param_id = 2;
                        for param in sig.params.as_slice() {
                            let field_name = format!("param_{}", param_id);
                            param_id += push_param(&mut code, param_id, param);
                            code.putfield(support_name, &field_name, param);
                            id += 1;
                        }

                        code.vreturn();
                    }

                    // write the id field
                    supporting.field(FFlags::Final | FFlags::Synthetic, "id", &Type::Int);

                    // write the rest of the parameters in the supporting class
                    let mut id = 0;
                    for param in sig.params.as_slice() {
                        let field_name = format!("param_{}", param_id);
                        supporting.field(FFlags::Final | FFlags::Synthetic, &field_name, param);
                        id += 1;
                    }

                    // write the class data
                    self.supporting
                        .insert(support_name.display_jni().to_string(), supporting.into_vec());
                }

                // load the parameters from the support class and call the super constructor
                code.astore(var_native_ret).aload(0);
                let mut id = 0;
                for param in native_sig.params.as_slice() {
                    let field_name = format!("param_{}", param_id);
                    code.aload(var_native_ret)
                        .getfield(support_name, &field_name, param);
                    id += 1;
                }
                code.invokespecial(&self.extends, "<init>", &native_sig)
                    .aload(0)
                    .aload(var_native_ret)
                    .getfield(support_name, "id", &Type::Int)
                    .putfield(&self.name, "id", &Type::Int);
            }
        }

        // call all late init functions
        for func in late_init {
            code.aload(0)
                .invokevirtual(&self.name, func, &MethodSig::void(&[]));
        }

        // cleanup
        code.vreturn();
    }

    pub fn export_field(&mut self, access: EnumSet<FFlags>, name: &str, ty: &Type) {
        self.class.field(access, name, ty);
    }

    pub fn export_native(&mut self, name: &str, sig: &MethodSig, is_static: bool) {
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
        sig: &MethodSig,
        native_name: &str,
        native_sig: &MethodSig,
        has_id_param: bool,
    ) {
        let mut method = self.class.method(access, name, sig);
        let mut code = method.code();

        // retrieve parameters
        let native_sig_params = if has_id_param {
            &native_sig.params[1..]
        } else {
            &native_sig.params
        };

        // validate parameters
        if has_id_param {
            assert_eq!(native_sig_params[0], Type::Int);
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
            code.getfield(&self.name, self.id_param, &Type::Int);
        }
        for param in native_sig_params {
            param_id += push_param(&mut code, param_id, &param);
        }
        if access.contains(MFlags::Static) {
            code.invokestatic(&self.name, native_name, native_sig);
        } else {
            code.invokevirtual(&self.name, native_name, native_sig);
        }
        return_param(&mut code, &sig.ret_ty);
    }

    pub fn into_vec(self) -> Vec<(String, Vec<u8>)> {
        let mut data = Vec::new();
        data.push((self.name.display_jni().to_string(), self.class.into_vec()));
        data.extend(self.supporting);
        data
    }
}
