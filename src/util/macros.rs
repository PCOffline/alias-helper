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
/// ```ignore
/// unwrap_or_panic!(Result, ErrorCode)
/// ```
///
/// This unwrap ignores the `err` value received in the closure, so it's important to use only ErroCodes that don't take an `err`` parameter.
///
/// ## Example
/// ```ignore
/// enum ErrorCode<'a> {
///   Foo,
///   Bar(&'a str)
/// }
///
/// let foo = Ok(5);
/// let bar = Err("oh no");
///
/// let five = unwrap_or_panic!(foo, ErrorCode::Foo); // Unwraps successfully with the value 5
/// let panics = unwrap_or_panic!(bar, ErrorCode::Bar("some data")); // Calls ErrorCode::Bar.log and panics with the defined error message.
/// ```
#[allow(unused_macros)]
macro_rules! unwrap_or_panic {
    ($result:expr, $error_code:expr) => {{
        $result.unwrap_or_else(|_| $error_code.log_and_panic(function_name!()))
    }};
}

/// Attempts to unwrap a given result, otherwise logs and panics with the given ErrorCode and the received error.
/// ## Syntax
/// ```ignore
/// unwrap_or_panic_err!(Result, ErrorCode)
/// ```
///
/// Unlike [unwrap_or_log], this macro passes the `err` value from the closure to the ErrorCode tuple.
///
/// ### Example
/// ```ignore
/// enum ErrorCode<'a> {
///   Five,
///   Foo(&'a str, ::std::io::Error),
/// }
///
/// let success = Ok(5);
/// let foo = std::fs::File::open("nonexistent_file.txt");
///
/// let five = unwrap_or_panic_err!(success, ErrorCode::Five); // Unwraps successfully with the value 5
/// let panics = unwrap_or_panic_err!(foo, ErrorCode::Foo, "some data"); // Passes the std::io::Error as the 2nd argument; calls ErrorCode::Foo.log, and panics with the default error message
/// ```
///
macro_rules! unwrap_or_panic_err {
    ($result:expr, $error_variant:path $(, $arg:expr)* ) => {{
        $result.unwrap_or_else(|err| $error_variant($($arg),*, err).log_and_panic(function_name!()))
    }};
}

macro_rules! debug_value {
    ($($var:ident),+ $(,)?) => {
        $(
            debug!("[{}] {} is {:?}", function_name!(), stringify!($var), $var);
        )+
    };
}

pub(crate) use debug_value;
pub(crate) use function_name;
#[allow(unused_imports)]
pub(crate) use unwrap_or_panic;
pub(crate) use unwrap_or_panic_err;
