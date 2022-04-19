use crate::{conversions::JavaConversionOwned, JniEnv};
use jni::objects::JObject;
use std::marker::PhantomData;

pub struct JArray<'env, T: JavaConversionOwned<'env>> {
    env: JniEnv<'env>,
    obj: JObject<'env>,
    _phantom: PhantomData<T>,
}
impl<'env, T: JavaConversionOwned<'env>> JArray<'env, T> {
    pub(crate) fn from_obj(env: JniEnv<'env>, obj: JObject<'env>) -> Self {
        JArray { env, obj, _phantom: Default::default() }
    }
    pub(crate) fn obj(&self) -> JObject<'env> {
        self.obj
    }

    // TODO
}
