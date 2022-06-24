#[cfg(test)]
mod tests;

use chacha20poly1305::aead;
use core::fmt;
use std::array::TryFromSliceError;
use std::{error, io, result};

pub type Result<T> = result::Result<T, Error>;

pub struct Error {
    repr: Repr,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

enum Repr {
    Simple(ErrorKind),
    SimpleMessage(ErrorKind, &'static &'static str),
    Custom(Box<Custom>),
}

type GenericError = dyn error::Error + Send + Sync;
type GenericErrorStatic = dyn error::Error + Send + Sync + 'static;

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<GenericError>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    FileNotFound,
    FileAlreadyExist,
    WrongPassword,
    FileTooSmall,
    FileInvalidMagic,
    FileMetaDecodeError,
    IOError,
    OtherError,
}

impl ErrorKind {
    pub fn to_str(self) -> &'static str {
        use ErrorKind::*;
        match self {
            FileNotFound => "File not found",
            FileAlreadyExist => "File already exist",
            WrongPassword => "Wrong password",
            FileTooSmall => "File is too small",
            FileMetaDecodeError => "File's meta_raw decode error",
            FileInvalidMagic => "Invalid file magic",
            IOError => "IO Error",
            OtherError => "Unknown error",
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            repr: Repr::Simple(kind),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error {
            repr: Repr::Custom(Box::from(Custom {
                kind: ErrorKind::IOError,
                error: Box::from(err),
            })),
        }
    }
}

impl From<TryFromSliceError> for Error {
    fn from(err: TryFromSliceError) -> Self {
        Error {
            repr: Repr::Custom(Box::from(Custom {
                kind: ErrorKind::OtherError,
                error: Box::from(err),
            })),
        }
    }
}

impl From<aead::Error> for Error {
    fn from(err: aead::Error) -> Self {
        Error {
            repr: Repr::Custom(Box::from(Custom {
                kind: ErrorKind::WrongPassword,
                error: Box::from(err),
            })),
        }
    }
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<GenericError>>,
    {
        Self::_new(kind, error.into())
    }

    fn _new(kind: ErrorKind, error: Box<GenericError>) -> Self {
        Error {
            repr: Repr::Custom(Box::from(Custom { kind, error })),
        }
    }

    pub const fn new_const(
        kind: ErrorKind,
        message: &'static &'static str,
    ) -> Self {
        Error {
            repr: Repr::SimpleMessage(kind, message),
        }
    }

    #[inline]
    pub fn get_ref(&self) -> Option<&GenericErrorStatic> {
        match &self.repr {
            Repr::Simple(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(c) => Some(&*c.error),
        }
    }

    #[inline]
    #[must_use]
    pub fn into_inner(self) -> Option<Box<GenericError>> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(c) => Some(c.error),
        }
    }

    #[inline]
    #[must_use]
    pub fn get_mut(&mut self) -> Option<&mut GenericErrorStatic> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(ref mut c) => Some(&mut *c.error),
        }
    }

    #[inline]
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Simple(kind) => kind,
            Repr::SimpleMessage(kind, _) => kind,
            Repr::Custom(ref c) => c.kind,
        }
    }

    ///

    pub fn new_file_not_found(filename: &str) -> Self {
        Self::new(
            ErrorKind::FileNotFound,
            format!("File '{}' not found", filename),
        )
    }

    pub fn file_already_exist(filename: &str) -> Self {
        Self::new(
            ErrorKind::FileAlreadyExist,
            format!("File '{}' not found", filename),
        )
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Repr::Simple(kind) => fmt
                .debug_tuple("Kind")
                .field(&kind)
                .finish(),
            Repr::SimpleMessage(kind, &message) => fmt
                .debug_struct("Error")
                .field("kind", &kind)
                .field("message", &message)
                .finish(),
            Repr::Custom(c) => fmt::Debug::fmt(&c, fmt),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr {
            Repr::Simple(kind) => write!(fmt, "{}", kind.to_str()),
            Repr::SimpleMessage(_, &msg) => msg.fmt(fmt),
            Repr::Custom(ref c) => c.error.fmt(fmt),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(ref c) => c.error.source(),
        }
    }
}

fn _assert_error_is_sync_send() {
    fn _is_sync_send<T: Sync + Send>() {}
    _is_sync_send::<Error>();
}
