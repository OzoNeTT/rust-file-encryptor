use core::fmt;
use std::error;
use std::str::Utf8Error;

pub struct MetaError {
    pub repr: ErrorKind,
}

impl fmt::Debug for MetaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    WrongHeaderVecSize,
    WrongRawVecSize,
    WrongRawCipherKind,
    WrongEncryptedVecSize,
    WrongEncryptedWrongStringsAmount,
    WrongEncryptedUtf8Error,
}

impl ErrorKind {
    pub fn to_str(&self) -> &'static str {
        use ErrorKind::*;
        match self {
            WrongHeaderVecSize => "Wrong Header vector size", // TODO: allow to specify the size
            WrongRawVecSize => "Wrong Raw vector size", // TODO: allow to specify the size
            WrongRawCipherKind => "Wrong Raw cipher kind",
            WrongEncryptedVecSize => "Wrong Encrypted vector size", // TODO: allow to specify the size
            WrongEncryptedWrongStringsAmount => {
                "Wrong Encrypted strings amount"
            } // TODO: allow to specify the size
            WrongEncryptedUtf8Error => {
                "Wrong Encrypted string conversion error"
            } // TODO: allow to specify the size
        }
    }
}

impl From<ErrorKind> for MetaError {
    fn from(kind: ErrorKind) -> Self {
        MetaError { repr: kind }
    }
}

impl From<Utf8Error> for MetaError {
    fn from(_: Utf8Error) -> Self {
        MetaError {
            repr: ErrorKind::WrongEncryptedUtf8Error,
        }
    }
}

impl fmt::Display for MetaError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.repr.to_str())
    }
}

impl error::Error for MetaError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

fn _assert_error_is_sync_send() {
    fn _is_sync_send<T: Sync + Send>() {}
    _is_sync_send::<MetaError>();
}
