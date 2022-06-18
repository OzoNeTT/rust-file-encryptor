use std::mem::size_of;
use super::{Error, ErrorKind, Result};
use crate::error::Repr;


#[test]
fn test_size() {
    assert!(size_of::<Error>() <= size_of::<[usize; 2]>());
}

#[test]
fn error_with_string() {
    let number = 9;

    let err = Error::new_const(
        ErrorKind::FileNotFound,
        &"Amognus bebra sus",
    );

    let mut message = "error message";
    let err1 = Error::new(
        ErrorKind::WrongPassword,
        message.to_string(),
    );

    ;
}
