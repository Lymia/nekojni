use crate::{
    conversions::{JavaConversion, JavaConversionType},
    errors::*,
    objects::JavaClass,
    JniEnv,
};
use jni::{
    objects::{JObject, JValue},
    sys::jobject,
};
use nekojni_utils::constcat_generic;
use parking_lot::lock_api::RwLock;
use std::marker::PhantomData;

pub trait SyntheticTyType {
    const CLASS_NAME: &'static str;
}

pub struct SyntheticTy<'env, T: SyntheticTyType>(JObject<'env>, PhantomData<T>);
impl<'env, T: SyntheticTyType> SyntheticTy<'env, T> {
    pub fn new(this: JObject<'env>) -> Self {
        SyntheticTy(this, PhantomData)
    }
}
impl<'env, T: SyntheticTyType> JavaConversionType for SyntheticTy<'env, T> {
    type JavaType = jobject;
    const JNI_TYPE: &'static str = constcat_generic!("L", T::CLASS_NAME, ";");
}
unsafe impl<'env, T: SyntheticTyType> JavaConversion<'env> for SyntheticTy<'env, T> {
    fn to_java(&self, _: JniEnv<'env>) -> Self::JavaType {
        self.0.into_inner()
    }
    fn to_java_value(&self, _: JniEnv<'env>) -> JValue<'env> {
        JValue::Object(self.0)
    }
    unsafe fn from_java_ref<R>(
        _: Self::JavaType,
        _: JniEnv<'env>,
        _: impl FnOnce(&Self) -> R,
    ) -> R {
        unreachable!()
    }
    unsafe fn from_java_mut<R>(
        _: Self::JavaType,
        _: JniEnv<'env>,
        _: impl FnOnce(&mut Self) -> R,
    ) -> R {
        unreachable!()
    }
    fn null() -> Self::JavaType {
        std::ptr::null_mut()
    }
}

pub trait ConstructorReturnTy<'env, W: SyntheticTyType> {
    type ReturnType: JavaConversion<'env>;
    const RET_TY: &'static str;
    const SUPER_CTOR_SIGNATURE: &'static str;
    const HELPER_CTOR_SIGNATURE: &'static str;
    fn ctor_new(self, param_class: &str, env: JniEnv<'env>) -> Result<Self::ReturnType>;
}

impl<'env, T: JavaClass<'env>, W: SyntheticTyType> ConstructorReturnTy<'env, W> for T {
    type ReturnType = u32;
    const RET_TY: &'static str = "I";
    const SUPER_CTOR_SIGNATURE: &'static str = "()V";
    const HELPER_CTOR_SIGNATURE: &'static str = "()V"; // unused
    fn ctor_new(self, param_class: &str, env: JniEnv<'env>) -> Result<Self::ReturnType> {
        env.get_id_manager::<T>().allocate(RwLock::new(self))
    }
}
impl<'env, T: JavaClass<'env>, W: SyntheticTyType, P: JavaConversion<'env>>
    ConstructorReturnTy<'env, W> for (T, P)
{
    type ReturnType = SyntheticTy<'env, W>;
    const RET_TY: &'static str = constcat_generic!("L", W::CLASS_NAME, ";");
    const SUPER_CTOR_SIGNATURE: &'static str = constcat_generic!("(", P::JNI_TYPE, ")V");
    const HELPER_CTOR_SIGNATURE: &'static str = constcat_generic!("(I", P::JNI_TYPE, ")V");
    fn ctor_new(self, param_class: &str, env: JniEnv<'env>) -> Result<Self::ReturnType> {
        let id = env.get_id_manager::<T>().allocate(RwLock::new(self.0))?;
        Ok(SyntheticTy::new(env.new_object(
            param_class,
            <Self as ConstructorReturnTy<'env, W>>::HELPER_CTOR_SIGNATURE,
            &[id.to_java_value(env), P::to_java_value(&self.1, env)],
        )?))
    }
}
macro_rules! generate_tuples {
    ($(($ty:ident $id:tt))*) => {
        impl<'env, T: JavaClass<'env>, W: SyntheticTyType, $($ty: JavaConversion<'env>,)*>
            ConstructorReturnTy<'env, W> for (T, ($($ty,)*))
        {
            type ReturnType = SyntheticTy<'env, W>;
            const RET_TY: &'static str = constcat_generic!("L", W::CLASS_NAME, ";");
            const SUPER_CTOR_SIGNATURE: &'static str =
                constcat_generic!("(", $($ty::JNI_TYPE,)* ")V");
            const HELPER_CTOR_SIGNATURE: &'static str =
                constcat_generic!("(I", $($ty::JNI_TYPE,)* ")V");
            fn ctor_new(self, param_class: &str, env: JniEnv<'env>) -> Result<Self::ReturnType> {
                let id = env.get_id_manager::<T>().allocate(RwLock::new(self.0))?;
                Ok(SyntheticTy::new(env.new_object(
                    param_class,
                    <Self as ConstructorReturnTy<'env, W>>::HELPER_CTOR_SIGNATURE,
                    &[id.to_java_value(env), $($ty::to_java_value(&(self.1).$id, env),)*],
                )?))
            }
        }
    }
}
generate_tuples!((P0 0) (P1 1));
generate_tuples!((P0 0) (P1 1) (P2 2));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6) (P7 7));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6) (P7 7) (P8 8));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6) (P7 7) (P8 8) (P9 9));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6) (P7 7) (P8 8) (P9 9) (P10 10));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6) (P7 7) (P8 8) (P9 9) (P10 10) (P11 11));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6) (P7 7) (P8 8) (P9 9) (P10 10) (P11 11) (P12 12));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6) (P7 7) (P8 8) (P9 9) (P10 10) (P11 11) (P12 12) (P13 13));
generate_tuples!((P0 0) (P1 1) (P2 2) (P3 3) (P4 4) (P5 5) (P6 6) (P7 7) (P8 8) (P9 9) (P10 10) (P11 11) (P12 12) (P13 13) (P14 14));

impl<'env, T: ConstructorReturnTy<'env, W>, W: SyntheticTyType> ConstructorReturnTy<'env, W>
    for Result<T>
{
    type ReturnType = T::ReturnType;
    const RET_TY: &'static str = constcat_generic!("L", W::CLASS_NAME, ";");
    const SUPER_CTOR_SIGNATURE: &'static str = T::SUPER_CTOR_SIGNATURE;
    const HELPER_CTOR_SIGNATURE: &'static str = T::HELPER_CTOR_SIGNATURE;
    fn ctor_new(self, param_class: &str, env: JniEnv<'env>) -> Result<Self::ReturnType> {
        match self {
            Ok(v) => T::ctor_new(v, param_class, env),
            Err(e) => Err(e),
        }
    }
}
