use crate::error;
use crate::error::ErrorKind::FileInvalidCipherId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CipherKind {
    ChaCha20Poly1305,
    AesGcm,
}

impl CipherKind {
    pub fn to_u8(self) -> u8 {
        use CipherKind::*;
        match self {
            ChaCha20Poly1305 => 0,
            AesGcm => 1,
        }
    }

    pub fn to_str(self) -> &'static str {
        use CipherKind::*;
        match self {
            ChaCha20Poly1305 => "Cipher ChaCha20Poly1305",
            AesGcm => "Cipher AesGcm",
        }
    }
}

impl TryInto<CipherKind> for u8 {
    type Error = error::Error;

    fn try_into(self) -> Result<CipherKind, Self::Error> {
        match self {
            _ => Err(Self::Error::new(
                FileInvalidCipherId,
                format!("Cipher ID {} is invalid", self),
            )),
        }
    }
}
