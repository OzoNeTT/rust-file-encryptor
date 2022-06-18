#[cfg(test)]
mod tests;

use chacha20poly1305::aead;
use core::fmt;
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

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    FileNotFound,
    WrongPassword,
    IOError,
}

impl ErrorKind {
    pub fn to_str(&self) -> &'static str {
        use ErrorKind::*;
        match *self {
            FileNotFound => "File not found",
            WrongPassword => "Wrong password",
            IOError => "IOError",
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

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::_new(kind, error.into())
    }

    fn _new(
        kind: ErrorKind,
        error: Box<dyn error::Error + Send + Sync>,
    ) -> Self {
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
    pub fn get_ref(
        &self,
    ) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
        match &self.repr {
            Repr::Simple(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(c) => Some(&*c.error),
        }
    }

    #[inline]
    pub fn into_inner(self) -> Option<Box<dyn error::Error + Send + Sync>> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(c) => Some(c.error),
        }
    }

    #[inline]
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Simple(kind) => kind,
            Repr::SimpleMessage(kind, _) => kind,
            Repr::Custom(ref c) => c.kind,
        }
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
