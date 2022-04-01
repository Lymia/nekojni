use crate::{
    conversions::{JavaConversion, JavaReturnConversion},
    errors::*,
    jni_env::JniEnv,
};
use std::{any::Any, panic::AssertUnwindSafe};

pub trait MethodReturn<T> {
    fn into_inner(self) -> T;
    fn is_error(&self) -> bool;
    fn emit_error(self, env: JniEnv, exception_class: &str) -> Result<()>;
}

impl<T> MethodReturn<T> for T {
    fn into_inner(self) -> T {
        self
    }
    fn is_error(&self) -> bool {
        false
    }
    fn emit_error(self, _env: JniEnv, _exception_class: &str) -> Result<()> {
        Err(Error::message("attempted to emit error from method that cannot fail"))
    }
}
impl<T, E: ErrorTrait + 'static> MethodReturn<T> for StdResult<T, E> {
    fn into_inner(self) -> T {
        self.expect("internal error: into_inner called on Err")
    }
    fn is_error(&self) -> bool {
        self.is_err()
    }

    #[inline(never)]
    fn emit_error(self, env: JniEnv, exception_class: &str) -> Result<()> {
        let err = self.err().expect("internal error: emit_error called on Ok");
        Error::wrap(err).emit_error(env, exception_class)
    }
}
impl<T> MethodReturn<T> for Result<T> {
    fn into_inner(self) -> T {
        self.expect("internal error: into_inner called on Err")
    }
    fn is_error(&self) -> bool {
        self.is_err()
    }

    #[inline(never)]
    fn emit_error(self, env: JniEnv, exception_class: &str) -> Result<()> {
        let err = self.err().expect("internal error: emit_error called on Ok");
        err.emit_error(env, exception_class)
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
        eprintln!("Aborting due to fatal error...");
        std::process::abort(); // rip
    }
}

pub fn catch_panic<R>(func: impl FnOnce() -> R) -> Result<R> {
    match std::panic::catch_unwind(AssertUnwindSafe(func)) {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::panicked(get_panic_string(e))),
    }
}

/// The function that handles the return value of object methods, and prevents panics from crossing
/// the FFI barrier.
#[allow(deprecated)]
pub fn catch_panic_jni<'env, T: JavaReturnConversion<'env>, R: MethodReturn<T>>(
    env: JniEnv<'env>,
    func: impl FnOnce() -> R,
) -> T::JavaRetType {
    // for safety, just in case there's a bug that might cause panics in e.g. backtrace, since
    // we invoke a lot of weird stuff trying to get the panic string.
    match std::panic::catch_unwind(AssertUnwindSafe(|| {
        let exception_class = crate::internal::globals::get_default_exception_class();
        match catch_panic(|| {
            let result = func();
            if result.is_error() {
                check_fail(env.emit_error(env, exception_class));
                T::null_ret()
            } else {
                result.into_inner().to_java_ret(env)
            }
        }) {
            Ok(v) => v,
            Err(e) => {
                check_fail(e.emit_error(env, exception_class));
                T::null_ret()
            }
        }
    })) {
        Ok(v) => v,
        Err(_) => std::process::abort(),
    }
}
