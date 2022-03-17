use jni::JNIEnv;
use std::{
    backtrace::Backtrace,
    borrow::Cow,
    fmt::{Display, Formatter},
    panic::Location,
};
use thiserror::Error;

// internal reexports
pub use std::{error::Error as ErrorTrait, result::Result as StdResult};

/// The error type used for `nekojni`.
///
/// This error does not implement [`Error`](`ErrorTrait`) to allow a `From` implementation for any
/// standard error.
#[derive(Debug)]
pub struct Error(Box<ErrorData>);

#[derive(Debug)]
struct ErrorData {
    location: &'static Location<'static>,
    data: ErrorType,
    backtrace: Option<Backtrace>,
    override_except_class: Option<Cow<'static, str>>,
}

#[derive(Error, Debug)]
enum ErrorType {
    #[error("Internal error: {0}")]
    Wrapped(#[source] Box<dyn ErrorTrait + 'static>),
    #[error("Internal error: {0}")]
    Error(Cow<'static, str>),
    #[error("{0}")]
    Message(Cow<'static, str>),
    #[error("Rust code panicked: {0}")]
    Panicking(Cow<'static, str>),
}
impl ErrorType {
    fn has_backtrace(&self) -> bool {
        if let Some(bt) = ErrorTrait::source(self) {
            bt.backtrace().is_some()
        } else {
            false
        }
    }
    fn is_validation_message(&self) -> bool {
        match self {
            ErrorType::Message(_) => true,
            _ => false,
        }
    }
}

impl Error {
    #[inline(never)]
    #[track_caller]
    fn raw_new(tp: ErrorType, request_backtrace: bool) -> Self {
        let backtrace = if tp.has_backtrace() || !request_backtrace {
            None
        } else {
            Some(Backtrace::capture())
        };
        Error(Box::new(ErrorData {
            location: Location::caller(),
            data: tp,
            backtrace,
            override_except_class: None,
        }))
    }

    /// Creates a new `Error` with an internal error message.
    #[inline(never)]
    #[track_caller]
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::raw_new(ErrorType::Error(msg.into()), true)
    }

    /// Creates a new `Error` with an error message.
    ///
    /// Unlike [`Error::new`], this does not record a backtrace, and will directly use the text
    /// as the cause of any generated exception.
    #[inline(never)]
    #[track_caller]
    pub fn message(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::raw_new(ErrorType::Message(msg.into()), false)
    }

    /// Creates a new `Error` from a Rust panic.
    #[inline(never)]
    #[track_caller]
    pub(crate) fn panicked(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::raw_new(ErrorType::Panicking(msg.into()), false)
    }

    /// Wraps any error in an `Error`.
    #[inline(never)]
    #[track_caller]
    pub fn wrap<T: ErrorTrait + 'static>(err: T) -> Self {
        Self::raw_new(ErrorType::Wrapped(Box::new(err)), true)
    }

    /// Catches a panic and converts it to an `Error`.
    pub fn catch_panic<R>(func: impl FnOnce() -> R) -> Result<R> {
        crate::panicking::catch_panic(func)
    }

    /// Emits an error into an [`JNIEnv`]
    pub fn emit_error(&self, env: &JNIEnv, exception_class: &str) -> Result<()> {
        let class = match &self.0.override_except_class {
            Some(x) => x,
            None => exception_class,
        };
        env.throw_new(class, self.to_string())?;
        Ok(())
    }

    /// Sets the class used when emitting this error as an exception to JNI.
    ///
    /// The class is given as an JNI internal name.
    #[inline(never)]
    pub fn set_exception_class(mut self, class: impl Into<Cow<'static, str>>) -> Self {
        self.0.override_except_class = Some(class.into());
        self
    }

    /// Returns the cause of this error.
    pub fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
        ErrorTrait::source(&self.0.data)
    }

    /// Returns the backtrace for this error.
    pub fn backtrace(&self) -> Option<&Backtrace> {
        if let Some(bt) = &self.0.backtrace {
            Some(bt)
        } else if let Some(source) = ErrorTrait::source(&self.0.data) {
            source.backtrace()
        } else {
            None
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.data.is_validation_message() {
            Display::fmt(&self.0.data, f)
        } else {
            write!(
                f,
                "{} (at {}:{})",
                self.0.data,
                self.0.location.file(),
                self.0.location.line()
            )
        }
    }
}
impl<T: ErrorTrait + 'static> From<T> for Error {
    fn from(t: T) -> Self {
        Error::wrap(t)
    }
}

/// The result type used for `nekojni`.
pub type Result<T> = StdResult<T, Error>;

/// Returns from the current function with an internal [`struct@Error`].
///
/// This requires the function return a [`Result`], and uses the same format as [`format!`].
#[macro_export]
macro_rules! jni_bail {
    ($($tt:tt)*) => {
        return $crate::__macro_internals::std::result::Result::Err(
            $crate::Error::new($crate::__macro_internals::std::format!($($tt)*))
        )
    }
}

/// Returns from the current function with an internal [`struct@Error`], if a precondition fails.
///
/// This requires the function return a [`Result`], and uses the same format as [`assert!`].
#[macro_export]
macro_rules! jni_assert {
    ($condition:expr, $($tt:tt)*) => {
        if !$condition {
            jni_bail!($($tt)*)
        }
    }
}

/// Returns from the current function with an [`struct@Error`]. Use this function for exceptions
/// that are meant to be thrown directly to Java code.
///
/// This requires the function return a [`Result`], and uses the same format as [`format!`].
///
/// Optionally, you may add an initial argument starting with @ to set the exception class. For
/// example:
///
/// ```rust
/// # use nekojni::*;
/// # fn test() -> Result<u32> {
/// jni_throw!(@"com/example/SomeExceptionClass", "This function encountered an error!")
/// # }
/// ```
#[macro_export]
macro_rules! jni_throw {
    (@ $exception_class:literal) => {
        return $crate::__macro_internals::std::result::Result::Err(
            $crate::Error::new($crate::__macro_internals::std::format!(""))
                .set_exception_class($exception_class)
        )
    };
    (@ $exception_class:literal, $($tt:tt)*) => {
        return $crate::__macro_internals::std::result::Result::Err(
            $crate::Error::new($crate::__macro_internals::std::format!($($tt)*))
                .set_exception_class($exception_class)
        )
    };
    ($($tt:tt)*) => {
        return $crate::__macro_internals::std::result::Result::Err(
            $crate::Error::new($crate::__macro_internals::std::format!($($tt)*))
        )
    };
}
