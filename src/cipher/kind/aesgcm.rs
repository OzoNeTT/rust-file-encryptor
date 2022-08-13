use crate::cipher::kind::Cipher;
use crate::{not_implemented, EncryptedMeta};
use std::io::{Read, Write};

pub struct AesGcm {
    size: Option<usize>,
}

impl AesGcm {
    pub fn new(size: Option<usize>) -> Self {
        return Self { size };
    }
}

impl Cipher for AesGcm {
    fn encrypt(
        &self,
        mut source: Box<dyn Read>,
        mut target: Box<dyn Write>,
        key: &[u8; 32],
        nonce: &[u8],
        enc_meta: &EncryptedMeta,
    ) -> crate::error::Result<()> {
        Err(not_implemented!("AesGcm::encrypt"))
    }

    fn decrypt(
        &self,
        mut source: Box<dyn Read>,
        mut target: Box<dyn Write>,
        key: &[u8; 32],
        nonce: &[u8],
    ) -> crate::error::Result<EncryptedMeta> {
        Err(not_implemented!("AesGcm::decrypt"))
    }
}
