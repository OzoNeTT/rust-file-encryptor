#[macro_export]
macro_rules! not_implemented {
    () => ($crate::error::Error::new_const(
        $crate::error::ErrorKind::OtherError,
        &"Not implemented",
    ));
    ($($arg:tt)+) => ($crate::error::Error::new(
        $crate::error::ErrorKind::OtherError,
        format!("Not implemented: {}", format_args!($($arg)+))
    ));
}
