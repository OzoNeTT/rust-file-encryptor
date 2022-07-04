use crate::cipher::kind::Cipher;
use crate::meta_enc::parser::MetaEncryptedDynamicParser;
use crate::utils::DynamicParser;
use crate::{not_implemented, EncryptedMeta};
use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use std::io::{Read, Write};

pub struct ChaCha20 {
    size: Option<usize>,
}

impl ChaCha20 {
    pub fn new(size: Option<usize>) -> Self {
        return Self { size };
    }
}

impl Cipher for ChaCha20 {
    fn encrypt(
        &self,
        source: &mut dyn Read,
        target: &mut dyn Write,
        key: &[u8; 32],
        nonce: &[u8],
        enc_meta: &EncryptedMeta,
    ) -> crate::error::Result<()> {
        let aead = XChaCha20Poly1305::new(key.as_ref().into());

        let mut stream_encryptor =
            stream::EncryptorBE32::from_aead(aead, nonce.into());

        const BUFFER_LEN: usize = 500;

        let mut glob_len = 0;

        // let meta_enc =
        //     stream_encryptor.encrypt_next(enc_meta.to_vec().as_slice())?;
        // target.write_all(&meta_enc)?;

        let mut meta_enc_enc: bool = false;

        loop {
            let mut buffer = if meta_enc_enc { vec![0u8; BUFFER_LEN] } else {
                vec![0u8; BUFFER_LEN - enc_meta.len()]
            };

            let mut read_count = source.read(&mut buffer)?;
            glob_len += read_count;
            match self.size {
                None => {
                    println!("Encrypting {:?}", glob_len);
                }
                Some(size) => {
                    println!("Encrypting {:?}/{:?}", glob_len, size, );
                }
            }
            println!("Old Buffer to encrypt: {buffer:?}");

            if !meta_enc_enc {
                let mut vec_meta = enc_meta.to_vec();
                vec_meta.extend_from_slice(&buffer[..read_count]);
                read_count += enc_meta.len();
                buffer = vec_meta;
                meta_enc_enc = true;
            }

            println!("Buffer to encrypt: {buffer:?}");

            if read_count == BUFFER_LEN {
                println!("Encrypt next");
                let ciphertext =
                    stream_encryptor.encrypt_next(buffer.as_slice())?;

                target.write_all(&ciphertext)?;
            } else {
                println!("Encrypt last");
                let ciphertext =
                    stream_encryptor.encrypt_last(&buffer[..read_count])?;
                target.write_all(&ciphertext)?;
                break;
            }
        }

        Ok(())
    }

    fn decrypt(
        &self,
        source: &mut dyn Read,
        target: &mut dyn Write,
        key: &[u8; 32],
        nonce: &[u8],
    ) -> crate::error::Result<EncryptedMeta> {
        let aead = XChaCha20Poly1305::new(key.as_ref().into());
        let mut stream_decryptor =
            stream::DecryptorBE32::from_aead(aead, nonce.into());

        const BUFFER_LEN: usize = 500 + 16;
        let mut glob_len = 0usize;

        let mut meta_enc_parser = MetaEncryptedDynamicParser::new();

        loop {
            let mut buffer = [0u8; BUFFER_LEN];
            let mut read_count = source.read(&mut buffer)?;
            println!("Buffer to decrypt: {buffer:?}");

            glob_len += read_count;

            match self.size {
                None => {
                    println!("Encrypting {:?}", glob_len);
                }
                Some(size) => {
                    println!("Encrypting {:?}/{:?}", glob_len, size, );
                }
            }

            let slice = &buffer[..read_count];

            if read_count == BUFFER_LEN {
                println!("Decrypt next");

                let plain_text = stream_decryptor.decrypt_next(slice)?;
                if meta_enc_parser.ready() {
                    target.write_all(&plain_text)?;
                } else {
                    let result = meta_enc_parser.parse_next(&plain_text)?;
                    target.write_all(result)?;
                }
            } else {
                println!("Decrypt last");

                let plain_text = stream_decryptor.decrypt_last(slice)?;
                println!("Plain text: {plain_text:?}");

                if meta_enc_parser.ready() {
                    target.write_all(&plain_text)?;
                } else {
                    let result = meta_enc_parser.parse_next(&plain_text)?;
                    target.write_all(result)?;
                }
                break;
            }
        }

        Ok(meta_enc_parser.try_into()?)
    }
}
