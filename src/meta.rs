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
    pub filename: String,
}

impl PartialEq<Self> for EncryptedMeta {
    fn eq(&self, other: &Self) -> bool {
        self.magic == other.magic
            && self.nonce == other.nonce
            && self.filename == other.filename
    }
}

impl EncryptedMeta {
    pub const MAGIC: [u8; MAGIC_SIZE] = [0x52, 0x46, 0x45, 0x44];

    pub fn new(nonce: &[u8; 19], filename: &str) -> Self {
        Self {
            magic: EncryptedMeta::MAGIC,
            filename: filename.to_string(),
            nonce: *nonce,
        }
    }

    pub fn len(&self) -> usize {
        MAGIC_SIZE + self.filename.len() + NONCE_SIZE + 2
    }

    pub fn to_vec(&self) -> Vec<u8> {
        vec![0u8]
            .into_iter()
            .chain(self.filename.bytes())
            .chain([0u8])
            .chain(self.nonce)
            .chain(self.magic)
            .collect::<Vec<u8>>()
    }

    pub fn from_vec(vec: &[u8]) -> error::Result<Self> {
        if vec.len() <= META_MIN_SIZE {
            return Err(ErrorKind::FileTooSmall.into());
        }
        if vec[vec.len() - MAGIC_SIZE..] != Self::MAGIC {
            return Err(ErrorKind::FileInvalidMagic.into());
        }

        let nonce = array_ref![
            vec[vec.len() - (MAGIC_SIZE + NONCE_SIZE)..vec.len() - 4],
            0,
            19
        ];

        let str_end = vec
            .iter()
            .rev()
            .skip(MAGIC_SIZE + NONCE_SIZE + 1)  // +one zero char
            ;
        let str_result = str_end
            .clone()
            .map_while(|c| match *c != b'\x00' {
                true => Some(*c),
                false => None,
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

        let filename = from_utf8(str_result.as_slice()).unwrap_or("");
        println!(
            "Filename {:?}",
            from_utf8(str_result.as_slice())
        );

        Ok(EncryptedMeta::new(nonce, filename))
    }

    pub fn is_valid_encoded(vec: &[u8]) -> bool {
        vec[vec.len() - MAGIC_SIZE..] == Self::MAGIC
    }
}

///////////////////////////////////////////////////////////////////
///////////////////////////    TESTS    ///////////////////////////
///////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::error;
    use crate::meta::EncryptedMeta;

    const NONCE: [u8; 19] = [
        10u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 0u8, 1u8, 2u8, 3u8,
        4u8, 5u8, 6u8, 7u8, 8u8,
    ];

    #[test]
    fn to_vec() {
        let meta = EncryptedMeta::new(&NONCE, "file.txt");

        assert_eq!(
            meta.to_vec().as_slice(),
            b"\x00file.txt\x00\x0A\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08RFED",
        );
    }

    #[test]
    fn from_vec() -> error::Result<()> {
        let a = b"\x00file.txt\x00\x0A\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08RFED".to_vec();
        let result = EncryptedMeta::from_vec(&a)?;

        assert_eq!(
            result,
            EncryptedMeta::new(&NONCE, "file.txt"),
        );

        Ok(())
    }
}
