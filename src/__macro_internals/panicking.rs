use crate::{__macro_internals::ClassInfo, errors::*, JavaConversion};
use jni::JNIEnv;
use std::{any::Any, panic::AssertUnwindSafe};

pub trait MethodReturn<T> {
    fn into_inner(self) -> T;
    fn is_error(&self) -> bool;
    fn emit_error(self, env: &JNIEnv, exception_class: &str) -> Result<()>;
}
impl<T> MethodReturn<T> for T {
    fn into_inner(self) -> T {
        self
    }
    fn is_error(&self) -> bool {
        false
    }
    fn emit_error(self, _env: &JNIEnv, _exception_class: &str) -> Result<()> {
        Err(Error::message(
            "attempted to emit error from method that cannot fail",
        ))
    }
}
impl<T, E: ErrorTrait + 'static> MethodReturn<T> for StdResult<T, E> {
    fn into_inner(self) -> T {
        self.expect("internal error: into_inner called on Err")
    }
    fn is_error(&self) -> bool {
        self.is_err()
    }
    fn emit_error(self, env: &JNIEnv, exception_class: &str) -> Result<()> {
        let err = self.err().expect("internal error: emit_error called on Ok");
        let t_ptr = &err as &(dyn ErrorTrait + 'static);
        if let Some(err) = t_ptr.downcast_ref::<Error>() {
            err.emit_error(env, exception_class)
        } else {
            Error::wrap(err).emit_error(env, exception_class)
        }
    }
}

#[inline(never)]
fn get_panic_string(e: Box<dyn Any + Send + 'static>) -> String {
    if let Some(_) = e.downcast_ref::<String>() {
        match e.downcast::<String>() {
            Ok(s) => *s,
            Err(_) => "error retrieving string???".to_string(),
        }
    } else if let Some(s) = e.downcast_ref::<&'static str>() {
        s.to_string()
    } else {
        "could not retrieve panic data".to_string()
    }
}
#[inline(never)]
fn check_fail(r: Result<()>) {
    if let Err(e) = r {
        eprintln!("Error throwing native exception: {e:?}");
        eprintln!("[ aborting ]");
        std::process::abort(); // rip
    }
}
pub fn catch_panic_jni<T: JavaConversion, R: MethodReturn<T>>(
    env: &JNIEnv,
    func: impl FnOnce() -> R,
    class_info: &ClassInfo,
) -> T::JavaType {
    match std::panic::catch_unwind(AssertUnwindSafe(|| {
        let result = func();
        if result.is_error() {
            check_fail(env.emit_error(env, class_info.exception_class));
            T::null()
        } else {
            result.into_inner().into_java(env)
        }
    })) {
        Ok(v) => v,
        Err(e) => {
            let err_obj = Error::panicked(get_panic_string(e));
            check_fail(err_obj.emit_error(env, class_info.exception_class));
            T::null()
        }
    }
}
