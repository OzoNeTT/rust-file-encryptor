#[cfg(test)]
mod tests;

use crate::cipher::CipherKind;
use crate::meta::error::{ErrorKind as MetaErrorKind, MetaError};

pub const META_RAW_NONCE_SIZE: usize = 19usize;

/// Raw (non-encrypted)
/// Meta-information about file encryption
///
/// # Binary structure
///
/// - `CK` stands for Cipher Kind
/// - `--` stands for Reserved
/// - `N` stands for Nonce
///
/// ```kotlin
///      0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
/// 0x00 CK -- -- -- -- -- -- -- -- -- -- -- -- N  N  N
/// 0x10 N  N  N  N  N  N  N  N  N  N  N  N  N  N  N  N
/// ```
///
/// ## Example
///
/// Consider:
/// - the CipherKind is 0x01
/// - nonce is `0x01 ... 0x13`
///
/// ```kotlin
///      0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
/// 0x00 01 -- -- -- -- -- -- -- -- -- -- -- -- 01 02 03
/// 0x10 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F 10 11 12 13
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RawMeta {
    /// Cipher type
    pub cipher_kind: CipherKind,

    /// Public number for a cipher
    pub nonce: [u8; META_RAW_NONCE_SIZE],
}

impl RawMeta {
    const NONCE_BYTE_INDEX: usize = Self::size() - META_RAW_NONCE_SIZE;

    pub const fn size() -> usize {
        0x20
    }

    pub const fn len(&self) -> usize {
        Self::size()
    }

    pub const fn is_empty(&self) -> bool {
        false
    }

    pub const fn version() -> u8 {
        1
    }

    pub fn to_bytes(&self) -> [u8; Self::size()] {
        let mut result: [u8; Self::size()] = [0u8; Self::size()];
        result[0] = self.cipher_kind.to_u8();

        (&mut result[Self::NONCE_BYTE_INDEX..]).clone_from_slice(&self.nonce);

        log::trace!(target: "meta/raw RawMeta to_bytes", "Result: {result:?}");
        result
    }

    pub fn try_from_bytes(
        bytes: [u8; Self::size()],
    ) -> Result<Self, MetaError> {
        Ok(Self {
            cipher_kind: bytes[0].try_into()?,
            nonce: bytes[Self::NONCE_BYTE_INDEX..]
                .try_into()
                .expect("Wrong slice size"),
        })
    }
}

impl TryFrom<[u8; Self::size()]> for RawMeta {
    type Error = MetaError;

    fn try_from(value: [u8; Self::size()]) -> Result<RawMeta, Self::Error> {
        RawMeta::try_from_bytes(value)
    }
}

impl TryFrom<&Vec<u8>> for RawMeta {
    type Error = MetaError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        log::debug!(target: "meta/raw RawMeta try_from", "value.len(): {:?}",  value.len());

        RawMeta::try_from_bytes(
            value
                .as_slice()
                .try_into()
                .map_err(|_| MetaError::from(MetaErrorKind::WrongRawVecSize))?,
        )
    }
}

impl TryFrom<Vec<u8>> for RawMeta {
    type Error = MetaError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}
