use super::{Error, ErrorKind, Result};
use crate::error::{Custom, Repr};
use std::mem::size_of;
use std::{error, fmt};

#[test]
fn test_size() {
    assert!(size_of::<Error>() <= size_of::<[usize; 2]>());
}

#[test]
fn test_debug_error() {
    let err = Error {
        repr: Repr::Custom(Box::from(Custom {
            kind: ErrorKind::FileNotFound,
            error: Box::from(Error {
                repr: super::Repr::SimpleMessage(
                    ErrorKind::IOError,
                    &"File thefile.txt not found",
                ),
            }),
        })),
    };

    let expected = format!(
        "Custom {{ \
        kind: FileNotFound, \
        error: Error {{ \
            kind: IOError, \
            message: \"File thefile.txt not found\" \
        }} \
        }}"
    );

    assert_eq!(format!("{:?}", err), expected);
}

#[test]
fn test_downcasting() {
    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("asdf")
        }
    }

    impl error::Error for TestError {}

    let mut err = Error::new(ErrorKind::IOError, TestError);
    assert!(err.get_ref().unwrap().is::<TestError>());
    assert_eq!(
        "asdf",
        err.get_ref().unwrap().to_string()
    );
    assert!(err.get_mut().unwrap().is::<TestError>());

    let extracted = err.into_inner().unwrap();
    extracted
        .downcast::<TestError>()
        .unwrap();
}

#[test]
fn test_const() {
    const E: Error = Error::new_const(ErrorKind::WrongPassword, &"hello");

    assert_eq!(E.kind(), ErrorKind::WrongPassword);
    assert_eq!(E.to_string(), "hello");
    assert!(format!("{:?}", E).contains("\"hello\""));
    assert!(format!("{:?}", E).contains("WrongPassword"));
}

#[test]
fn test_result() {
    fn inner_fun() -> Result<()> {
        Err(Error::new(
            ErrorKind::WrongPassword,
            "Wrong password",
        ))
    }

    assert_eq!(inner_fun().is_err(), true);
    assert_eq!(
        inner_fun().expect_err("").kind(),
        ErrorKind::WrongPassword
    );
}
