use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::{cmp, io};
use std::io::ErrorKind;
use arrayref::array_ref;
use std::str;
use std::str::{from_utf8, Utf8Error};

pub const MAGIC_SIZE: usize = 4usize;
pub const NONCE_SIZE: usize = 19usize;

const META_MIN_SIZE: usize = MAGIC_SIZE + NONCE_SIZE + 2;


#[derive(Debug)]
pub struct EncryptedMeta {
    magic: [u8; MAGIC_SIZE],
    nonce: [u8; NONCE_SIZE],
    filename: String,

}

impl PartialEq<Self> for EncryptedMeta {
    fn eq(&self, other: &Self) -> bool {
        self.magic == other.magic && self.nonce == other.nonce && self.filename == other.filename
    }
}

impl EncryptedMeta {
    pub const MAGIC: [u8; MAGIC_SIZE] = [0x52, 0x46, 0x45, 0x44];

    pub fn new(
        nonce: &[u8; NONCE_SIZE],
        filename: &str,
    ) -> Self {
        return Self {
            magic: EncryptedMeta::MAGIC,
            filename: filename.into(),
            nonce: *nonce,
        };
    }

    pub fn to_vec(&self) -> Vec<u8> {
        vec![0u8].into_iter()
            .chain(self.filename.bytes())
            .chain(self.nonce)
            .chain(self.magic)
            .collect::<Vec<u8>>()
    }


    pub fn from_vec(vec: &Vec<u8>) -> io::Result<Self> {
        if vec.len() <= META_MIN_SIZE {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid length"));
        }
        if vec[vec.len() - MAGIC_SIZE..] != Self::MAGIC {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid magic"));
        }

        let nonce = array_ref![vec[vec.len() - (MAGIC_SIZE + NONCE_SIZE)..vec.len() - 4], 0, 19];

        let str_end = vec.into_iter().rev().skip(MAGIC_SIZE + NONCE_SIZE);
        let str_result = str_end
            .clone()
            .map_while(|c| match *c != b'\x00' {
                true => Some(*c),
                false => None
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
        ;

        let filename = match from_utf8(str_result.as_slice()){
            Ok(str) => &str,
            Err(_) => ""
        };
        println!("Filename {:?}", from_utf8(str_result.as_slice()));

        return Ok(EncryptedMeta::new(&nonce, filename));
    }

    pub fn load_file(){
        return;
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
            meta.to_vec().as_slice(),
            b"\x00file.txt\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08RFED",
        );
    }

    #[test]
    fn from_vec() -> io::Result<()> {
        let a = b"\x00file.txt\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08RFED".to_vec();
        let result = EncryptedMeta::from_vec(&a)?;

        assert_eq!(
            result,
            EncryptedMeta::new(&NONCE, "file.txt"),
        );

        Ok(())
    }
}