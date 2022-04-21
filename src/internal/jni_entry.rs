use crate::{
    conversions::{JavaConversionType, JavaReturnConversion, JniAbiType},
    errors::*,
    jni_env::JniEnv,
};
use jni::JNIEnv;
use std::{any::Any, panic::AssertUnwindSafe};

pub trait MethodReturn {
    type Intermediate;
    type ReturnTy: JniAbiType;
    const JNI_RETURN_TYPE: &'static str;
    fn into_inner(self) -> Self::Intermediate;
    fn is_error(&self) -> bool;
    fn emit_error(self, env: JniEnv, exception_class: &str) -> Result<()>;
}

impl<T: JavaConversionType> MethodReturn for T {
    type Intermediate = T;
    type ReturnTy = T::JavaType;
    const JNI_RETURN_TYPE: &'static str = T::JNI_TYPE;
    fn into_inner(self) -> T {
        self
    }
    fn is_error(&self) -> bool {
        false
    }
    #[inline(never)]
    fn emit_error(self, _env: JniEnv, _exception_class: &str) -> Result<()> {
        Err(Error::message("attempted to emit error from method that cannot fail"))
    }
}
impl<T: JavaConversionType, E: ErrorTrait + 'static> MethodReturn for StdResult<T, E> {
    type Intermediate = T;
    type ReturnTy = T::JavaType;
    const JNI_RETURN_TYPE: &'static str = T::JNI_TYPE;
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
impl<T: JavaConversionType> MethodReturn for Result<T> {
    type Intermediate = T;
    type ReturnTy = T::JavaType;
    const JNI_RETURN_TYPE: &'static str = T::JNI_TYPE;
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
#[cold]
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
#[cold]
pub fn panic_abort(e: Box<dyn Any + Send + 'static>) -> ! {
    std::panic::catch_unwind(AssertUnwindSafe(|| {
        println!("Panic encountered in nekojni internal code: {}", get_panic_string(e));
    }))
    .ok();
    std::process::abort();
}

#[inline(never)]
#[cold]
pub fn fail(e: Error) -> ! {
    eprintln!("Error thrown by internal nekojni code: {e}");
    eprintln!("Aborting due to fatal error...");
    std::process::abort(); // rip
}

#[inline(never)]
#[cold]
fn check_fail(r: Result<()>) {
    if let Err(e) = r {
        fail(e);
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
///
/// This has the weird name it does to allow us to identify it in the stack trace (for purposes of
/// passing a stack trace cleanly into Java code).
#[inline(never)]
pub fn __njni_entry_point<R: MethodReturn, F: FnOnce(JniEnv) -> R>(
    env: JNIEnv,
    func: F,
    exception_class: &str,
) -> <R::Intermediate as JavaConversionType>::JavaType
where
    for<'a> R::Intermediate: JavaReturnConversion<'a>,
{
    // for safety, just in case there's a bug that might cause panics in e.g. backtrace, since
    // we invoke a lot of weird stuff trying to get the panic string.
    match std::panic::catch_unwind(AssertUnwindSafe(|| {
        match JniEnv::with_env(env, |env| {
            match catch_panic(|| {
                let result = func(env);
                if result.is_error() {
                    check_fail(result.emit_error(env, exception_class));
                    R::Intermediate::null_ret()
                } else {
                    result.into_inner().to_java_ret(env)
                }
            }) {
                Ok(v) => Ok(v),
                Err(e) => {
                    check_fail(e.emit_error(env, exception_class));
                    Ok(R::Intermediate::null_ret())
                }
            }
        }) {
            Ok(v) => v,
            Err(e) => fail(e),
        }
    })) {
        Ok(v) => v,
        Err(e) => panic_abort(e),
    }
}
