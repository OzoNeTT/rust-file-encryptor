use crate::{error, CipherKind, EncryptedMeta};
use std::io;

mod aesgcm;
mod chacha20;

pub trait Cipher {
    fn encrypt(
        &self,
        source: Box<dyn io::Read>,
        target: Box<dyn io::Write>,
        key: &[u8; 32],
        nonce: &[u8],
        enc_meta: &EncryptedMeta,
    ) -> error::Result<()>;

    fn decrypt(
        &self,
        source: Box<dyn io::Read>,
        target: Box<dyn io::Write>,
        key: &[u8; 32],
        nonce: &[u8],
    ) -> error::Result<EncryptedMeta>;
}

pub fn select_cipher(
    kind: CipherKind,
    original_size: Option<usize>,
) -> Box<dyn Cipher> {
    match kind {
        CipherKind::ChaCha20Poly1305 => {
            Box::from(chacha20::ChaCha20::new(original_size)) as Box<dyn Cipher>
        }
        CipherKind::AesGcm => {
            Box::from(aesgcm::AesGcm::new(original_size)) as Box<dyn Cipher>
        }
    }
}
