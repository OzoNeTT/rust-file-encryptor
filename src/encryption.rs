// use core::slice::SlicePattern;

use crate::meta_enc::EncryptedMeta;
use crate::{error, error::ErrorKind, OpenOrCreate};
use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

pub fn try_parse(source_file: &Path) -> error::Result<bool> {
    let mut buff = vec![];

    let valid_enc: bool;

    {
        let mut file = File::open(source_file)?;

        // TODO: buffered read the end of the file
        // (do not read all file)
        file.read_to_end(&mut buff)?;
        valid_enc = EncryptedMeta::is_valid_encoded(&buff);
    }

    Ok(valid_enc)
}

pub fn get_raw_meta(source_file: &Path) -> error::Result<EncryptedMeta> {
    let mut buff = vec![];

    let mut file = File::open_read_only(source_file)?;
    file.read_to_end(&mut buff)?;

    (&buff).try_into()
}

#[allow(dead_code)]
pub fn get_and_remove_meta(source_file: &Path) -> error::Result<EncryptedMeta> {
    let meta_info = get_meta(source_file)?;
    let meta_size = meta_info.len();

    let file = File::open_write(source_file)?;
    let new_len = file.metadata()?.len() - meta_size as u64;
    file.set_len(new_len)?;

    Ok(meta_info)
}

pub fn add_raw_meta(
    meta: EncryptedMeta,
    target_file: &mut dyn Write,
) -> error::Result<()> {
    target_file.write_all(meta.to_vec().as_slice())?;

    Ok(())
}

pub fn decrypt_file(
    source_file_path: &Path,
    dist_file_path: &Path,
    meta_length: usize,
    key: &[u8; 32],
    nonce: &[u8],
    preview: bool,
) -> Result<(), error::Error> {
    let mut source_file = File::open_read_only(source_file_path)?;
    let mut dist_file: Box<dyn Write> = if preview {
        Box::new(io::stdout())
    } else {
        Box::new(File::open_or_create(dist_file_path)?)
    };

    if dist_file_path.exists() {
        return Err(error::Error::file_already_exist(
            dist_file_path.to_str().unwrap_or(""),
        ));
    }

    let aead = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream_decryptor =
        stream::DecryptorBE32::from_aead(aead, nonce.into());

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

        println!(
            "Decrypting {:?}/{:?}",
            glob_len, file_size_nometa
        );
        //println!("Buffer: {:?}", &buffer[..read_count]);

        let slice = &buffer[..read_count];

        if glob_len >= file_size_nometa {
            let ciphertext = stream_decryptor.decrypt_last(slice)?;
            dist_file.write_all(&ciphertext)?;
            break;
        } else {
            let ciphertext = stream_decryptor.decrypt_next(slice)?;
            dist_file.write_all(&ciphertext)?;
        }
    }

    Ok(())
}

pub fn encrypt_file(
    source: &mut dyn io::Read,
    target: &mut dyn io::Write,
    key: &[u8; 32],
    nonce: &[u8],
    enc_meta: &EncryptedMeta,
) -> Result<(), error::Error> {
    // let mut source_file = File::open(source_file_path)?;
    //
    // if dist_file_path.exists() {
    //     return Err(error::Error::file_already_exist(
    //         dist_file_path.to_str().unwrap_or(""),
    //     ));
    // }

    // {
    // Truncate the file
    // let dist_file = File::open_or_create(dist_file_path)?;
    // dist_file.set_len(0)?;
    // }

    // let mut dist_file = File::open_or_create(dist_file_path)?;

    let aead = XChaCha20Poly1305::new(key.as_ref().into());

    let mut stream_encryptor =
        stream::EncryptorBE32::from_aead(aead, nonce.into());

    const BUFFER_LEN: usize = 500;

    let mut glob_len = 0;
    let file_len = source_file.metadata()?.len() as usize;

    target.write_all(&stream_encryptor.encrypt_next(enc_meta.to_vec())?)?;

    loop {
        let mut buffer = [0u8; BUFFER_LEN];
        let read_count = source.read(&mut buffer)?;
        glob_len += read_count;
        println!(
            "Encrypting {:?}/{:?}",
            glob_len, file_len,
        );

        if read_count == BUFFER_LEN {
            let ciphertext =
                stream_encryptor.encrypt_next(buffer.as_slice())?;

            dist_file.write_all(&ciphertext)?;
        } else {
            let ciphertext =
                stream_encryptor.encrypt_last(&buffer[..read_count])?;
            dist_file.write_all(&ciphertext)?;
            break;
        }
    }

    Ok(())
}
