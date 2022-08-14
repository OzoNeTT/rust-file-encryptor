#[cfg(test)]
mod tests;

use crate::meta::error::{ErrorKind as MetaErrorKind, MetaError};
use std::str::from_utf8;

const ENC_META_MIN_SIZE: usize = 1;

/// Encrypted
/// Meta-information about file encryption
///
/// # Binary structure
///
/// No static sized fields stored.
/// May contain zero bytes after strings (for an alignment)
///
/// Strings is being stored like in ELF files :)
/// Number of strings: 1.
///
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EncryptedMeta {
    /// Original filename
    pub filename: String,
}

impl EncryptedMeta {
    pub const fn version() -> u8 {
        1
    }

    pub fn len(&self) -> usize {
        self.filename.len() + 1
    }

    pub fn is_empty(&self) -> bool {
        self.filename.len() == 0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::<u8>::with_capacity(self.len())
            .into_iter()
            .chain(self.filename.bytes())
            .chain([0u8])
            .collect::<Vec<u8>>()
    }

    pub fn try_from_bytes(value: &[u8]) -> Result<Self, MetaError> {
        println!(
            "try_from_bytes value.len() {}",
            value.len()
        );

        if value.len() < ENC_META_MIN_SIZE {
            return Err(MetaErrorKind::WrongEncryptedVecSize.into());
        }
        if value[value.len() - 1] != 0x00 {
            return Err(MetaErrorKind::WrongEncryptedWrongStringsAmount.into());
        }

        let res = value
            .into_iter()
            .skip(
                0, /* structure body before strings */
            )
            .map_while(|c| if *c != 0x00 { Some(*c) } else { None })
            .collect::<Vec<u8>>();
        Ok(Self {
            filename: from_utf8(res.as_slice())?.to_string(),
        })
    }
}

impl TryFrom<&Vec<u8>> for EncryptedMeta {
    type Error = MetaError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_bytes(value)
    }
}

impl TryFrom<Vec<u8>> for EncryptedMeta {
    type Error = MetaError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_bytes(&value)
    }
}
