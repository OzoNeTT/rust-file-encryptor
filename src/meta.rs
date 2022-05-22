use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::{cmp, io};
use std::io::ErrorKind;
use arrayref::array_ref;


#[derive(Debug)]
pub struct EncryptedMeta {
    magic: [u8; 4],
    nonce: [u8; 19],
    filename: String,

}

impl PartialEq<Self> for EncryptedMeta {
    fn eq(&self, other: &Self) -> bool {
        self.magic == other.magic && self.nonce == other.nonce && self.filename == other.filename
    }
}



impl EncryptedMeta {
    pub const MAGIC: [u8; 4] = [0x52, 0x46, 0x45, 0x44];

    pub fn new(nonce: &[u8; 19], filename: &str) -> EncryptedMeta {
        return EncryptedMeta {
            magic: EncryptedMeta::MAGIC,
            filename: filename.into(),
            nonce: *nonce,
        };
    }

    pub fn to_vec(self) -> Vec<u8> {
        vec![0u8].into_iter()
            .chain(self.filename.bytes())
            .chain(self.nonce)
            .chain(self.magic)
            .collect::<Vec<u8>>()
    }

    pub fn from_vec(vec: &Vec<u8>) -> io::Result<EncryptedMeta> {
        if vec.len() <= 19 + 4 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid length"));
        }
        if vec[vec.len() - 4..] != EncryptedMeta::MAGIC {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid magic"));
        }

        let nonce = array_ref![vec[vec.len() - 19 - 4..vec.len() - 4], 0, 19];

        // TODO
        // FIXME : parse filename
        // TODO
        return Ok(EncryptedMeta::new(&nonce, "amongus"));
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use crate::meta::EncryptedMeta;

    const NONCE: [u8; 19] = [0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8];

    #[test]
    fn to_vec() {
        let meta = EncryptedMeta::new(&NONCE, "file.txt");

        assert_eq!(
            b"\x00file.txt\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08RFED",
            meta.to_vec().as_slice()
        );
    }

    #[test]
    fn from_vec() -> io::Result<()> {
        let a = b"\x00file.txt\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08RFED".to_vec();
        let result = EncryptedMeta::from_vec(&a)?;

        assert_eq!(
            EncryptedMeta::new(&NONCE, "file.txt"),
            result
        );

        Ok(())
    }
}