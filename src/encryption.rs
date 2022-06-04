// use core::slice::SlicePattern;

use std::{io};
use std::fs::{File};
use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use chacha20poly1305::{aead::{stream, NewAead}, XChaCha20Poly1305};
use crate::meta::{EncryptedMeta};
use crate::OpenOrCreate;



pub fn try_parse(
    source_file: &Path,
) -> io::Result<bool> {

    let mut buff = vec![];

    let mut valid_enc: bool;
    {
        let mut file = File::open(source_file)?;
        file.read_to_end(&mut buff)?;
        valid_enc = EncryptedMeta::is_valid_encoded(&buff);
    }
    Ok(valid_enc)
}

pub fn get_and_remove_meta(
    source_file: &Path,
) -> io::Result<EncryptedMeta> {

    let mut buff = vec![];

    let mut meta_info: EncryptedMeta;
    let mut meta_size = 0usize;

    {
        let mut file = File::open(source_file)?;
        file.read_to_end(&mut buff)?;

        meta_info = EncryptedMeta::from_vec(&buff)?;
        meta_size = meta_info.len();
    }

    {
        let mut file = File::open_write(source_file)?;
        let new_len = file.metadata()?.len() - meta_size as u64;
        file.set_len(new_len).expect(
            "Bruh bruh bruh"
        );
    }

    Ok(meta_info)
}

pub fn append_meta(
    nonce: &[u8; 19],
    source_file: &Path,
) -> io::Result<()> {
    let filename =  match source_file.to_str() {
        Some(str) => Ok(str),
        None => Err(io::Error::new(ErrorKind::Other, "I don't know u dumb maybe idk"))
    }?;

    let meta_info = EncryptedMeta::new(
        nonce,
        filename
    );
    let mut file = File::open_append(source_file)?;
    file.write(
        meta_info.to_vec().as_slice()
    ).expect("Aboba message here!");

    Ok(())
}

pub fn decrypt_file(
    source_file_path: &Path,
    dist_file_path: &Path,
    key: &[u8; 32],
    nonce: &[u8],
) -> io::Result<bool> {
    let mut source_file = File::open(source_file_path)?;
    let mut dist_file = File::open_or_create(dist_file_path)?;

    let aead = XChaCha20Poly1305::new(key.as_ref().into());

    let mut stream_decryptor = stream::DecryptorBE32::from_aead(
        aead,
        nonce.into(),
    );


    const BUFFER_LEN: usize = 500 + 16;
    let mut glob_len = 0;

    loop {
        let mut buffer = [0u8; BUFFER_LEN];
        let read_count = source_file.read(&mut buffer)?;
        glob_len += read_count;
        println!("Decrypting {:?}/{:?}", glob_len, source_file.metadata()?.len());
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

    Ok(true)
}

pub fn encrypt_file(
    source_file_path: &Path,
    dist_file_path: &Path,
    key: &[u8; 32],
    nonce: &[u8],
) -> io::Result<bool> {
    let mut source_file = File::open(source_file_path)?;
    let mut dist_file = File::open_or_create(dist_file_path)?;


    let aead = XChaCha20Poly1305::new(key.as_ref().into());

    let mut stream_encryptor = stream::EncryptorBE32::from_aead(
        aead,
        nonce.into(),
    );

    const BUFFER_LEN: usize = 500;

    let mut glob_len = 0;

    loop {
        let mut buffer = [0u8; BUFFER_LEN];
        let read_count = source_file.read(&mut buffer)?;
        glob_len += read_count;
        println!("Encrypting {:?}/{:?}", glob_len, source_file.metadata()?.len());
        if read_count == BUFFER_LEN {
            let ciphertext = stream_encryptor
                .encrypt_next(buffer.as_slice())
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, format!("Encrypting large file: {0}", err)))?;

            //println!("Ciphertext length: {}", ciphertext.len());
            dist_file.write(&ciphertext)?;
        } else {
            let ciphertext = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, format!("Encrypting large file: {0}", err)))?;
            dist_file.write(&ciphertext)?;
            break;
        }
    }

    Ok(true)
}