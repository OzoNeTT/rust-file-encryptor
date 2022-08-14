use crate::cipher::kind::Cipher;
use crate::{not_implemented, EncryptedMeta};
use std::io::{Read, Write};

#[allow(dead_code)]
pub struct AesGcm {
    size: Option<usize>,
}

impl AesGcm {
    pub fn new(size: Option<usize>) -> Self {
        Self { size }
    }
}

impl Cipher for AesGcm {
    fn encrypt(
        &self,
        _: Box<dyn Read>,
        _: Box<dyn Write>,
        _: &[u8; 32],
        _: &[u8],
        _: &EncryptedMeta,
    ) -> crate::error::Result<()> {
        Err(not_implemented!("AesGcm::encrypt"))
    }

    fn decrypt(
        &self,
        _: Box<dyn Read>,
        _: Box<dyn Write>,
        _: &[u8; 32],
        _: &[u8],
    ) -> crate::error::Result<EncryptedMeta> {
        Err(not_implemented!("AesGcm::decrypt"))
    }
}
