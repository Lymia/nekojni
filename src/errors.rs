use crate::{conversions::JavaConversion, jni_env::JniEnv};
use backtrace::Backtrace;
use jni::objects::JThrowable;
use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    panic::Location,
};
use thiserror::Error;

// internal reexports
pub use std::{error::Error as ErrorTrait, result::Result as StdResult};

// TODO: Extract the internal Java exception from Java errors, if at all possible.

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
    fn is_validation_message(&self) -> bool {
        match self {
            ErrorType::Message(_) => true,
            _ => false,
        }
    }
}

impl Error {
    #[inline(never)]
    #[cold]
    #[track_caller]
    fn raw_new(tp: ErrorType, request_backtrace: bool) -> Self {
        let backtrace = if !request_backtrace {
            None
        } else {
            Some(Backtrace::new_unresolved())
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
    #[cold]
    #[track_caller]
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::raw_new(ErrorType::Error(msg.into()), true)
    }

    /// Creates a new `Error` with an error message.
    ///
    /// Unlike [`Error::new`], this does not record a backtrace, and will directly use the text
    /// as the cause of any generated exception.
    #[inline(never)]
    #[cold]
    #[track_caller]
    pub fn message(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::raw_new(ErrorType::Message(msg.into()), false)
    }

    /// Creates a new `Error` from a Rust panic.
    #[inline(never)]
    #[cold]
    #[track_caller]
    pub(crate) fn panicked(msg: impl Into<Cow<'static, str>>) -> Self {
        // TODO: Source a backtrace properly
        Self::raw_new(ErrorType::Panicking(msg.into()), false)
    }

    /// Wraps any error in an `Error`.
    #[inline(never)]
    #[cold]
    #[track_caller]
    pub fn wrap<T: ErrorTrait + 'static>(err: T) -> Self {
        Self::raw_new(ErrorType::Wrapped(Box::new(err)), true)
    }

    /// Catches a panic and converts it to an `Error`.
    pub fn catch_panic<R>(func: impl FnOnce() -> R) -> Result<R> {
        crate::internal::jni_entry::catch_panic(func)
    }

    /// Emits an error into an [`JniEnv`]
    #[inline(never)]
    pub fn emit_error(&self, env: JniEnv, exception_class: &str) -> Result<()> {
        let class = match &self.0.override_except_class {
            Some(x) => x,
            None => exception_class,
        };
        let exception =
            env.new_object(class, "(Ljava/lang/String;)V", &[self.to_string().to_java_value(env)])?;
        'register_exc: {
            if exception_class == class {
                if let Some(backtrace) = self.backtrace() {
                    let mut backtrace = backtrace.clone();
                    backtrace.resolve();

                    // gather a full list of symbols we wish to print to console
                    let mut any_symbols_found = false;
                    let mut target_frames = Vec::new();
                    'outer: for frame in backtrace.frames() {
                        target_frames.push(frame);
                        for symbol in frame.symbols() {
                            if let Some(symbol) = symbol.name() {
                                any_symbols_found = true;
                                if symbol.to_string().contains("__njni_entry_point") {
                                    break 'outer;
                                }
                            }
                        }
                    }

                    // we assume there's no debug information we can make much use of anyway
                    if !any_symbols_found {
                        env.call_method(
                            exception,
                            "addRustTraceLine",
                            "(Ljava/lang/String;)V",
                            &[format!(
                                "\tat native <unknown symbol> ({}:{})",
                                self.0.location.file(),
                                self.0.location.line(),
                            )
                            .to_java_value(env)],
                        )?;
                        break 'register_exc;
                    }

                    // print the Rust stack trace into the Java exception type
                    let mut from_loc_emitted = false;
                    for frame in &target_frames {
                        // use the file location from `Location` if there isn't any
                        let mut from_loc = if !from_loc_emitted {
                            from_loc_emitted = true;
                            format!(" ({}:{})", self.0.location.file(), self.0.location.line())
                        } else {
                            String::new()
                        };

                        // print out the relevant symbols from the debugging information
                        let mut frame_any_symbols = false;
                        for symbol in frame.symbols() {
                            if symbol.filename().is_none() && symbol.name().is_none() {
                                continue;
                            }

                            frame_any_symbols = true;

                            let symbol_name = match symbol.name() {
                                Some(name) => name.to_string(),
                                None => format!("<unknown symbol>"),
                            };
                            let file_loc = match (symbol.filename(), symbol.lineno()) {
                                (Some(file_name), Some(file_lineno)) => {
                                    format!(" ({}:{})", file_name.display(), file_lineno)
                                }
                                (Some(file_name), None) => format!(" ({})", file_name.display()),
                                _ => format!(""),
                            };

                            const EXCLUDE_PATTERNS: &'static [&'static str] = &[
                                "backtrace::",
                                "nekojni::errors::Error::raw_new::",
                                "nekojni::errors::Error::new::",
                                "nekojni::errors::Error::wrap::",
                            ];
                            if EXCLUDE_PATTERNS.iter().any(|x| symbol_name.starts_with(x)) {
                                if !from_loc.is_empty() {
                                    from_loc = String::new();
                                    from_loc_emitted = false;
                                }
                            }
                            if !file_loc.is_empty() {
                                from_loc = String::new();
                            }

                            env.call_method(
                                exception,
                                "addRustTraceLine",
                                "(Ljava/lang/String;)V",
                                &[format!("\tat native {symbol_name}{file_loc}{from_loc}")
                                    .to_java_value(env)],
                            )?;
                        }

                        // fallback for if we can't resolve this symbol
                        if !frame_any_symbols {
                            env.call_method(
                                exception,
                                "addRustTraceLine",
                                "(Ljava/lang/String;)V",
                                &[format!("\tat <unknown symbol> @ {:?}{from_loc}", frame.ip())
                                    .to_java_value(env)],
                            )?;
                        }
                    }
                }
            }
        }
        env.throw(JThrowable::from(exception))?;
        Ok(())
    }

    /// Sets the class used when emitting this error as an exception to JNI.
    ///
    /// The class is given as an JNI internal name.
    #[inline(never)]
    #[cold]
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
            write!(f, "{}", self.0.data)
        }
    }
}
impl<T: ErrorTrait + 'static> From<T> for Error {
    #[inline(always)]
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
/// that are meant to be thrown directly to Java code. Notably, unlike [`jni_bail!`] nor
/// [`jni_assert!`], this function does not store a stack trace.
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
            $crate::Error::message($crate::__macro_internals::std::format!(""))
                .set_exception_class($exception_class)
        )
    };
    (@ $exception_class:literal, $($tt:tt)*) => {
        return $crate::__macro_internals::std::result::Result::Err(
            $crate::Error::message($crate::__macro_internals::std::format!($($tt)*))
                .set_exception_class($exception_class)
        )
    };
    ($($tt:tt)*) => {
        return $crate::__macro_internals::std::result::Result::Err(
            $crate::Error::message($crate::__macro_internals::std::format!($($tt)*))
        )
    };
}
