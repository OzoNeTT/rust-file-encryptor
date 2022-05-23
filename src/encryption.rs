// use core::slice::SlicePattern;

use std::{io, iter};
use std::fs::{File, metadata};
use std::io::{ErrorKind, Read, Write};
use chacha20poly1305::{
    aead::{stream, Aead, NewAead},
    XChaCha20Poly1305,
    Key,
    Nonce,
    XNonce,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use crate::meta;

pub fn decrypt_file(
    source_file: &mut File,
    dist_file: &mut File,
    key: &[u8; 32],
    nonce: &[u8],
) -> io::Result<()> {
    let aead = XChaCha20Poly1305::new(Key::from_slice(key));

    let mut stream_decryptor = stream::DecryptorBE32::from_aead(
        aead,
        nonce.into()
    );


    const BUFFER_LEN: usize = 512;


    let mut glob_len = 0;

    loop {
        let mut buffer = [0u8; BUFFER_LEN];
        let read_count = source_file.read(&mut buffer)?;
        glob_len += read_count;
        println!("Decrypting {:?}/{:?}", glob_len, source_file.metadata().unwrap().len());
        if read_count == BUFFER_LEN {
            let ciphertext = stream_decryptor
                .decrypt_next(buffer.as_slice())
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, format!("Decrypting large file: {0}", err)))?;
            dist_file.write(&ciphertext)?;
        } else {
            let ciphertext = stream_decryptor
                .decrypt_last(&buffer[..read_count])
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, format!("Decrypting large file: {0}", err)))?;
            dist_file.write(&ciphertext)?;
            break;
        }
    }

    Ok(())
}

pub fn encrypt_file(
    source_file: &mut File,
    dist_file: &mut File,
    key: &[u8; 32],
    nonce: &[u8],
) -> io::Result<()> {
    let aead = XChaCha20Poly1305::new(Key::from_slice(key));

    let mut stream_encryptor = stream::EncryptorBE32::from_aead(
        aead,
        nonce.into()
    );

    const BUFFER_LEN: usize = 512;

    let mut glob_len = 0;

    loop {
        let mut buffer = [0u8; BUFFER_LEN];
        let read_count = source_file.read(&mut buffer)?;
        glob_len += read_count;
        println!("Encrypting {:?}/{:?}", glob_len, source_file.metadata().unwrap().len());
        if read_count == BUFFER_LEN {
            let ciphertext = stream_encryptor
                .encrypt_next(buffer.as_slice())
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, format!("Encrypting large file: {0}", err)))?;
            dist_file.write(&ciphertext)?;
        } else {
            let ciphertext = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, format!("Encrypting large file: {0}", err)))?;
            dist_file.write(&ciphertext)?;
            break;
            // write anything
        }
    }

    // TODO: put meta::EncryptedMeta

    Ok(())
}