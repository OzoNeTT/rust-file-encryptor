// use core::slice::SlicePattern;

use crate::meta::EncryptedMeta;
use crate::OpenOrCreate;
use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

pub fn try_parse(source_file: &Path) -> io::Result<bool> {
    let mut buff = vec![];

    let valid_enc: bool;
    {
        let mut file = File::open(source_file)?;
        file.read_to_end(&mut buff)?;
        valid_enc = EncryptedMeta::is_valid_encoded(&buff);
    }
    Ok(valid_enc)
}

pub fn get_meta(source_file: &Path) -> io::Result<EncryptedMeta> {
    let mut buff = vec![];

    let mut file = File::open_read_only(source_file)?;
    file.read_to_end(&mut buff)?;

    EncryptedMeta::from_vec(&buff)
}

#[allow(dead_code)]
pub fn get_and_remove_meta(source_file: &Path) -> io::Result<EncryptedMeta> {
    let meta_info = get_meta(source_file)?;
    let meta_size = meta_info.len();

    let file = File::open_write(source_file)?;
    let new_len = file.metadata()?.len() - meta_size as u64;
    file.set_len(new_len).expect("Bruh bruh bruh");

    Ok(meta_info)
}

pub fn append_meta(nonce: &[u8; 19], original_file: &Path, source_file: &Path) -> io::Result<()> {
    let filename = original_file
        .file_name()
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "I don't know u dumb maybe idk"))?
        .to_str()
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "I don't know u dumb maybe idk"))?;

    let meta_info = EncryptedMeta::new(nonce, filename);
    let mut file = File::open_append(source_file)?;
    file.write_all(meta_info.to_vec().as_slice())
        .expect("Aboba message here!");

    Ok(())
}

pub fn decrypt_file(
    source_file_path: &Path,
    dist_file_path: &Path,
    meta_length: usize,
    key: &[u8; 32],
    nonce: &[u8],
) -> io::Result<bool> {
    let mut source_file = File::open_read_only(source_file_path)?;
    let mut dist_file = File::open_or_create(dist_file_path)?;

    let aead = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream_decryptor = stream::DecryptorBE32::from_aead(aead, nonce.into());

    // let method = &StreamDecryptorType::decrypt_next;

    const BUFFER_LEN: usize = 500 + 16;
    let mut glob_len = 0usize;
    let file_size = source_file.metadata()?.len() as usize;
    let file_size_nometa = file_size - meta_length;

    //println!("file_size        = {:?}", file_size);
    //println!("file_size_nometa = {:?}", file_size_nometa);

    loop {
        let mut buffer = [0u8; BUFFER_LEN];
        let mut read_count = source_file.read(&mut buffer)?;
        //println!("read_count = {:?}", read_count);

        glob_len += read_count;
        if glob_len > file_size_nometa {
            let delta = glob_len - file_size_nometa;

            //println!("glob_len         = {:?}", glob_len);
            //println!("file_size_nometa = {:?}", file_size_nometa);
            glob_len = file_size_nometa;
            read_count -= delta;
            //println!("read_count'      = {:?}", read_count);
        }

        println!("Decrypting {:?}/{:?}", glob_len, file_size_nometa);
        //println!("Buffer: {:?}", &buffer[..read_count]);

        let slice = &buffer[..read_count];
        let err_handle = |err| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("Decrypting large file: {:?}", err),
            )
        };

        if glob_len >= file_size_nometa {
            let ciphertext = stream_decryptor.decrypt_last(slice).map_err(err_handle)?;
            dist_file.write_all(&ciphertext)?;
            break;
        } else {
            let ciphertext = stream_decryptor.decrypt_next(slice).map_err(err_handle)?;
            dist_file.write_all(&ciphertext)?;
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

    {
        // Truncate the file
        let dist_file = File::open_or_create(dist_file_path)?;
        dist_file.set_len(0)?;
    }

    let mut dist_file = File::open_or_create(dist_file_path)?;

    let aead = XChaCha20Poly1305::new(key.as_ref().into());

    let mut stream_encryptor = stream::EncryptorBE32::from_aead(aead, nonce.into());

    const BUFFER_LEN: usize = 500;

    let mut glob_len = 0;
    let file_len = source_file.metadata()?.len() as usize;

    loop {
        let mut buffer = [0u8; BUFFER_LEN];
        let read_count = source_file.read(&mut buffer)?;
        glob_len += read_count;
        println!("Encrypting {:?}/{:?}", glob_len, file_len);

        if read_count == BUFFER_LEN {
            let ciphertext = stream_encryptor
                .encrypt_next(buffer.as_slice())
                .map_err(|err| {
                    io::Error::new(
                        ErrorKind::InvalidData,
                        format!("Encrypting large file: {0}", err),
                    )
                })?;

            // println!("Ciphertext length: {}", ciphertext.len());
            dist_file.write_all(&ciphertext)?;
        } else {
            let ciphertext = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| {
                    io::Error::new(
                        ErrorKind::InvalidData,
                        format!("Encrypting large file: {0}", err),
                    )
                })?;
            dist_file.write_all(&ciphertext)?;
            break;
        }
    }

    Ok(true)
}
