#[cfg(test)]
mod tests;

use crate::cipher::CipherKind;
use crate::{error, error::ErrorKind};
use arrayref::array_ref;

pub const MAGIC_SIZE: usize = 4usize;
pub const NONCE_SIZE: usize = 19usize;

const META_MIN_SIZE: usize = MAGIC_SIZE + NONCE_SIZE + 2;

/// Meta-information about file encryption
///
/// # Binary structure
///
/// 1. Zero byte
/// 2. Filename
/// 3. Zero byte
/// 4. Nonce
/// 5. Magic
/// 6. EOF
///
/// ## Example
///
/// Consider filename is `file.txt`, nonce is 0x12, 0x13, ...
///
/// ```kotlin
///      0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
/// 0x00 *  *  *  *  *  *  *  *  *  00 f  i  l  e  .  t
/// 0x10 x  t  00 12 13 14 15 16 17 18 19 1A 1B 1C 1D 1E
/// 0x20 1F 20 21 22 23 24 52 46 45 44 -  -  -  -  -  -
/// ```
#[derive(Debug)]
pub struct RawMeta {
    /// Magic number.
    /// Is being used for determining encrypted file
    pub magic: [u8; MAGIC_SIZE],

    /// Cipher type
    pub cipher_kind: CipherKind,

    /// Public number for a cipher
    pub nonce: [u8; NONCE_SIZE],
}

impl PartialEq<Self> for RawMeta {
    fn eq(&self, other: &Self) -> bool {
        self.magic == other.magic && self.nonce == other.nonce
    }
}

impl RawMeta {
    pub const MAGIC: [u8; MAGIC_SIZE] = [0x52, 0x46, 0x45, 0x44];

    pub fn new(nonce: &[u8; 19], cipher_kind: CipherKind) -> Self {
        Self {
            magic: RawMeta::MAGIC,
            cipher_kind,
            //filename: filename.to_string(),
            nonce: *nonce,
        }
    }

    pub fn len(&self) -> usize {
        MAGIC_SIZE + 1 + NONCE_SIZE + 2
    }

    pub fn is_empty(&self) -> bool {
        //self.filename.len() == 0
        false
    }

    pub fn to_vec(&self) -> Vec<u8> {
        vec![0u8]
            .into_iter()
            .chain(self.magic)
            .chain(self.nonce)
            .chain([self.cipher_kind.to_u8()])
            //.chain([0u8])
            //.chain(self.filename.bytes())
            .collect::<Vec<u8>>()
    }

    pub fn is_valid_encoded(vec: &[u8]) -> bool {
        if vec.len() <= META_MIN_SIZE {
            false
        } else {
            vec[..MAGIC_SIZE] == Self::MAGIC
        }
    }
}

impl TryInto<RawMeta> for &[u8] {
    type Error = error::Error;

    fn try_into(self) -> Result<RawMeta, Self::Error> {
        if self.len() <= META_MIN_SIZE {
            return Err(ErrorKind::FileTooSmall.into());
        }
        if self[self.len() - MAGIC_SIZE..] != RawMeta::MAGIC {
            return Err(ErrorKind::FileInvalidMagic.into());
        }

        let nonce = array_ref![
            self[MAGIC_SIZE..MAGIC_SIZE + NONCE_SIZE],
            0,
            NONCE_SIZE
        ];

        let cipher: CipherKind =
            self[MAGIC_SIZE + NONCE_SIZE + 1].try_into()?;

        Ok(RawMeta::new(
            nonce, cipher, //                filename_str
        ))
    }
}

impl TryInto<RawMeta> for &Vec<u8> {
    type Error = error::Error;

    fn try_into(self) -> Result<RawMeta, Self::Error> {
        self.as_slice().try_into()
    }
}
