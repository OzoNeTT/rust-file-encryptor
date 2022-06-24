#[cfg(test)]
mod tests;

use crate::{error, error::ErrorKind};
use arrayref::array_ref;
use std::str;
use std::str::from_utf8;

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
pub struct EncryptedMeta {
    /// Magic number.
    /// Is being used for determining encrypted file
    pub magic: [u8; MAGIC_SIZE],

    /// Public number for a cipher
    pub nonce: [u8; NONCE_SIZE],

    /// Original filename
    //pub filename: String,
}

impl PartialEq<Self> for EncryptedMeta {
    fn eq(&self, other: &Self) -> bool {
        self.magic == other.magic
            && self.nonce == other.nonce
            //&& self.filename == other.filename
    }
}

impl EncryptedMeta {
    pub const MAGIC: [u8; MAGIC_SIZE] = [0x52, 0x46, 0x45, 0x44];

    pub fn new(nonce: &[u8; 19],
               //filename: &str
    ) -> Self {
        Self {
            magic: RawMeta::MAGIC,
            //filename: filename.to_string(),
            nonce: *nonce,
        }
    }

    pub fn len(&self) -> usize {
        MAGIC_SIZE +
            //self.filename.len() +
            NONCE_SIZE + 2
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

impl TryInto<EncryptedMeta> for &[u8] {
    type Error = error::Error;

    fn try_into(self) -> Result<EncryptedMeta, Self::Error> {
        if self.len() <= META_MIN_SIZE {
            return Err(ErrorKind::FileTooSmall.into());
        }
        if self[self.len() - MAGIC_SIZE..] != EncryptedMeta::MAGIC {
            return Err(ErrorKind::FileInvalidMagic.into());
        }

        let nonce = array_ref![
            self[self.len() - (MAGIC_SIZE + NONCE_SIZE)..self.len() - 4],
            0,
            19
        ];

        /*let filename = self
            .iter()
            .skip(MAGIC_SIZE + NONCE_SIZE + 1) // +one zero char
            .map_while(|c| match *c != b'\x00' {
                true => Some(*c),
                false => None,
            })
            .collect::<Vec<_>>();

        let filename_str = from_utf8(filename.as_slice()).map_err(|e| {
            error::Error::new(ErrorKind::FileMetaDecodeError, e)
        })?;
        println!(
            "Filename {:?}",
            from_utf8(filename.as_slice())
        );

        Ok(EncryptedMeta::new(nonce, filename_str))
    }
}

impl TryInto<EncryptedMeta> for &Vec<u8> {
    type Error = error::Error;

    fn try_into(self) -> Result<EncryptedMeta, Self::Error> {
        self.as_slice().try_into()
    }
}
