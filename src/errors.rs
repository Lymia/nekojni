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
#[derive(Debug)]
pub struct Error(Box<ErrorData>);

#[derive(Debug)]
struct ErrorData {
    location: &'static Location<'static>,
    data: ErrorType,
    backtrace: Option<Backtrace>,
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
    pub fn panicked(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::raw_new(ErrorType::Panicking(msg.into()), false)
    }

    /// Wraps any error in an `Error`.
    #[inline(never)]
    #[track_caller]
    pub fn wrap<T: ErrorTrait + 'static>(err: T) -> Self {
        Self::raw_new(ErrorType::Wrapped(Box::new(err)), true)
    }

    /// Emits an error into an [`JNIEnv`]
    pub fn emit_error(&self, env: &JNIEnv, exception_class: &str) -> Result<()> {
        env.throw_new(exception_class, self.to_string())?;
        Ok(())
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
impl ErrorTrait for Error {
    fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
        ErrorTrait::source(&self.0.data)
    }
    fn backtrace(&self) -> Option<&Backtrace> {
        if let Some(bt) = &self.0.backtrace {
            Some(bt)
        } else if let Some(source) = ErrorTrait::source(&self.0.data) {
            source.backtrace()
        } else {
            None
        }
    }
}

macro_rules! error_from {
    ($($ty:ty),* $(,)?) => {$(
        impl From<$ty> for Error {
            #[inline(never)]
            #[track_caller]
            fn from(err: $ty) -> Self {
                Error::wrap(err)
            }
        }
    )*};
}
error_from!(jni::errors::Error, jni::errors::JniError,);

/// The result type used for `nekojni`.
pub type Result<T> = StdResult<T, Error>;
