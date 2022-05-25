// use core::slice::SlicePattern;

use std::{fs, io, iter};
use std::convert::TryInto;
use std::fs::{File, metadata};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;
use chacha20poly1305::{
    aead::{stream, Aead, NewAead},
    XChaCha20Poly1305,
    Key,
    Nonce,
    XNonce,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use crate::meta::EncryptedMeta;
use crate::OpenOrCreate;
use std::ffi::OsStr;

pub fn get_and_remove_meta(
    data: &Vec<u8>,
    source_file: &mut Path,
) -> io::Result<EncryptedMeta> {
    let meta_info = EncryptedMeta::from_vec(data)?;

    let mut file = File::open(source_file)?;
    file.set_len(file.metadata()?.len() - meta_info.length).expect(
        "Bruh bruh bruh"
    );

    Ok(meta_info)
}

pub fn append_meta(
    nonce: &[u8; 19],
    source_file: &mut Path,
) -> io::Result<()> {
    let filename =  match source_file.to_str() {
        Some(str) => Ok(str),
        None => Err(io::Error::new(ErrorKind::Other, "I don't know u dumb maybe idk"))
    }?;

    let meta_info = EncryptedMeta::new(
        nonce,
        filename
    );
    let mut file = File::open(source_file)?;
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
) -> io::Result<()> {
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

    Ok(())
}

pub fn encrypt_file(
    source_file_path: &Path,
    dist_file_path: &Path,
    key: &[u8; 32],
    nonce: &[u8],
) -> io::Result<()> {
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

            println!("Ciphertext length: {}", ciphertext.len());
            dist_file.write(&ciphertext)?;
        } else {
            let ciphertext = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, format!("Encrypting large file: {0}", err)))?;
            dist_file.write(&ciphertext)?;
            break;
        }
    }

    Ok(())
}