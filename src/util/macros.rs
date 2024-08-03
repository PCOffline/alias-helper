macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}

/// Attempts to unwrap a given result, otherwise logs and panics with the given ErrorCode.
/// ## Syntax
/// ```no_run
/// #[macro_use]
/// unwrap_or_log!(Result, ErrorCode)
/// ```
///
/// This unwrap ignores the `err` value received in the closure, so it's important to use only ErroCodes that don't take an `err`` parameter.
///
/// ## Example
/// ```rust
/// # #[macro_use]
/// enum ErrorCode<'a> {
///   Foo,
///   Bar(&'a str),
///   Invalid(&'a str)
/// }
///
/// let foo = Ok(5);
/// let bar = Err("oh no");
///
/// let five = unwrap_or_log!(foo, ErrorCode::Foo); // Unwraps successfully with the value 5
/// let panics = unwrap_or_log!(bar, ErrorCode::Bar("some data")); // Calls ErrorCode::Bar.log and panics with the defined error message.
/// ```
macro_rules! unwrap_or_log {
    ($result:expr, $error_code:expr) => {{
        $result.unwrap_or_else(|_| $error_code.log(function_name!()))
    }};
}

/// Attempts to unwrap a given result, otherwise logs and panics with the given ErrorCode and the received error.
/// ## Syntax
/// ```no_run
/// unwrap_or_log_with_err!(Result, ErrorCode)
/// ```
///
/// Unlike [unwrap_or_log], this macro passes the `err` value from the closure to the ErrorCode tuple.
/// 
/// ## Example
/// ```rust
/// ``` 
///
macro_rules! unwrap_or_log_with_err {
    ($result:expr, $error_variant:path $(, $arg:expr)* ) => {{
        $result.unwrap_or_else(|err| $error_variant($($arg),*, err).log(function_name!()))
    }};
}

macro_rules! debug_value {
    ($($var:ident),+ $(,)?) => {
        $(
            println!("[{}] {} is {:?}", function_name!(), stringify!($var), $var);
        )+
    };
}

pub(crate) use function_name;
pub(crate) use unwrap_or_log;
pub(crate) use debug_value;
pub(crate) use unwrap_or_log_with_err;
