use crate::cipher::kind::Cipher;
use crate::meta::header::MetaHeader;
use crate::{not_implemented, EncryptedMeta, ErrorKind};
use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use std::cmp::{max, min};
use std::fs::read;
use std::io;
use std::io::{BufRead, Read, Write};

pub struct ChaCha20 {
    size: Option<usize>,
}

impl ChaCha20 {
    pub fn new(size: Option<usize>) -> Self {
        Self { size }
    }
}

struct CipherProcessing {
    target: Box<dyn Write>,
    enc_header: Option<MetaHeader>,
    enc_meta_data: Vec<u8>,
}

impl CipherProcessing {
    fn new(target: Box<dyn Write>) -> Self {
        Self {
            target,
            enc_header: None,
            enc_meta_data: Vec::with_capacity(128),
        }
    }

    fn process(&mut self, buffer: Vec<u8>) -> crate::error::Result<()> {
        let mut cursor = io::Cursor::new(buffer);

        if self.enc_header.is_none() {
            // TODO: buffer size must be always > header size
            let mut enc_header_data = vec![0u8; MetaHeader::size()];
            cursor.read_exact(&mut enc_header_data)?;
            log::trace!(target: "cipher/kind/chacha20 CipherProcessing process", "Meta header buffer: {:?}", enc_header_data);

            let header = (&enc_header_data).try_into()?;
            self.enc_header = Some(header);
            log::debug!(target: "cipher/kind/chacha20 CipherProcessing process", "Read meta header");
            log::trace!(target: "cipher/kind/chacha20 CipherProcessing process", "MetaHeader: {:?}", header);
        }
        let header = self
            .enc_header
            .expect("Got none header after none processing");

        let mut tmp_enc_meta_data =
            vec![0u8; (header.size as usize) - self.enc_meta_data.len()];
        let _ = cursor.read(&mut tmp_enc_meta_data)?;
        self.enc_meta_data
            .extend(tmp_enc_meta_data);

        io::copy(&mut cursor, &mut self.target)?;

        Ok(())
    }
}

impl Cipher for ChaCha20 {
    fn encrypt(
        &self,
        mut source: Box<dyn Read>,
        mut target: Box<dyn Write>,
        key: &[u8; 32],
        nonce: &[u8],
        enc_meta: &EncryptedMeta,
    ) -> crate::error::Result<()> {
        log::debug!(target: "cipher/kind/chacha20 ChaCha20 encrypt", "Begin");
        let aead = XChaCha20Poly1305::new(key.as_ref().into());

        let mut stream_encryptor =
            stream::EncryptorBE32::from_aead(aead, nonce.into());

        const BUFFER_LEN: usize = 500;
        let mut glob_len = 0;

        let header = MetaHeader {
            size: enc_meta.len() as u64,
            magic: MetaHeader::MAGIC,
            version: EncryptedMeta::version(),
        };
        log::trace!(target: "cipher/kind/chacha20 ChaCha20 encrypt", "MetaHeader: {:?}",header);

        let vec_enc = enc_meta.to_vec();
        let meta_vector: Vec<u8> = header
            .to_vec()
            .into_iter()
            .chain(vec_enc)
            .collect();
        let meta_vector_len = meta_vector.len();
        let mut meta_vector_pos = 0usize;

        loop {
            let mut buffer = Vec::<u8>::new();
            let meta_vec_delta = min(
                BUFFER_LEN,
                meta_vector_len - meta_vector_pos,
            );
            if meta_vec_delta != 0 {
                buffer = meta_vector
                    [meta_vector_pos..meta_vector_pos + meta_vec_delta]
                    .to_vec();
                meta_vector_pos += meta_vec_delta;
            }

            let mut inner_buffer = vec![0u8; BUFFER_LEN - meta_vec_delta];
            let mut read_count =
                source.read(&mut inner_buffer)? + meta_vec_delta;
            log::debug!(target: "cipher/kind/chacha20 ChaCha20 encrypt","Plain text length: {}", read_count);

            glob_len += read_count;
            match self.size {
                None => {
                    log::info!(target: "cipher/kind/chacha20 ChaCha20 encrypt", "Encrypting {:>6}", glob_len);
                }
                Some(size) => {
                    log::info!(target: "cipher/kind/chacha20 ChaCha20 encrypt", "Encrypting {:>6}/{:>6}", glob_len, size,);
                }
            }

            buffer.extend(inner_buffer);
            let slice = &buffer[..read_count];

            let ciphertext = stream_encryptor.encrypt_next(slice)?;
            log::trace!(target: "cipher/kind/chacha20 ChaCha20 encrypt","Ciphertext: {:?}", ciphertext);

            // TODO: maybe replace by BufWriter
            target.write_all(&ciphertext)?;
            log::debug!(target: "cipher/kind/chacha20 ChaCha20 encrypt","Ciphertext block written into the file");
            if read_count != BUFFER_LEN {
                break;
            }
        }

        Ok(())
    }

    fn decrypt(
        &self,
        mut source: Box<dyn Read>,
        mut target: Box<dyn Write>,
        key: &[u8; 32],
        nonce: &[u8],
    ) -> crate::error::Result<EncryptedMeta> {
        log::debug!(target: "cipher/kind/chacha20 ChaCha20 decrypt", "Begin");

        let aead = XChaCha20Poly1305::new(key.as_ref().into());
        let mut stream_decryptor =
            stream::DecryptorBE32::from_aead(aead, nonce.into());

        const BUFFER_LEN: usize = 500 + 16; // 16 is MAC code length
        let mut glob_len = 0usize;

        let mut processing = CipherProcessing::new(Box::from(target));

        loop {
            let mut buffer = [0u8; BUFFER_LEN];
            let mut read_count = source.read(&mut buffer)?;
            let slice = &buffer[..read_count];
            log::trace!(target: "cipher/kind/chacha20 ChaCha20 decrypt","Buffer to decrypt: {:?}", slice);

            glob_len += read_count;

            match self.size {
                None => {
                    log::info!(target: "cipher/kind/chacha20 ChaCha20 decrypt", "Decrypting {:>6}", glob_len);
                }
                Some(size) => {
                    log::info!(target: "cipher/kind/chacha20 ChaCha20 decrypt", "Decrypting {:>6}/{:>6}", glob_len, size,);
                }
            }

            let plain_text = stream_decryptor.decrypt_next(slice)?;
            log::debug!(target: "cipher/kind/chacha20 ChaCha20 decrypt","Plain text length: {}", plain_text.len());

            processing.process(plain_text)?;
            if read_count != BUFFER_LEN {
                break;
            }
        }

        log::trace!(target: "cipher/kind/chacha20 ChaCha20 decrypt", "Encrypted meta buffer: {:?}", processing.enc_meta_data);
        Ok(processing.enc_meta_data.try_into()?)
    }
}
