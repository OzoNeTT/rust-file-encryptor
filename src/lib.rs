pub mod cipher;
pub mod encryption;
#[allow(dead_code)]
pub mod error;
pub mod file;
pub mod meta_enc;
pub mod meta_raw;
pub mod utils;

use arrayref::array_ref;
use std::convert::TryInto;
use std::fs::File;
use std::iter;
use std::path::Path;

use rand::{thread_rng, Rng};

use crate::cipher::CipherKind;
use crate::encryption::{add_raw_meta, decrypt_file, encrypt_file};
use crate::error::ErrorKind;
use crate::file::OpenOrCreate;
use crate::meta_enc::EncryptedMeta;
use crate::meta_raw::RawMeta;
use file::GetFileDirectory;
use rand::distributions::Alphanumeric;
use sha2::{Digest, Sha256};

extern crate core;
extern crate log;

pub fn get_hash(key: &str) -> error::Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());

    let hashed_key: [u8; 32] = hasher
        .finalize()
        .as_slice()
        .try_into()?;
    Ok(hashed_key)
}

pub fn try_decrypt(
    file_path: &Path,
    hash_from_key: [u8; 32],
    preview: bool,
) -> error::Result<()> {
    //target_file.seek(SeekFrom::Start(MAGIC_STRING.len() as u64))?;
    // let meta = get_meta(file_path)?;
    // let nonce = meta.nonce;
    //
    // //   .\\filename.txt
    // //   ./source.\\filename
    // //   ./source/.\\filename
    //
    // let filename = &meta.filename;
    // //let decrypt_file_path = &file_path.to_owned()
    // //    .with_file_name(filename);
    //
    // let decrypt_file_path = file_path.file_dir()?.join(filename);
    //
    // println!(
    //     "decrypt_file_path: {:?}",
    //     decrypt_file_path
    // );
    // decrypt_file(
    //     file_path,
    //     &decrypt_file_path,
    //     meta.len(),
    //     // 0,
    //     &hash_from_key,
    //     &nonce,
    //     preview,
    // )?;

    Err(error::Error::new_const(
        ErrorKind::OtherError,
        &"Not implemented",
    ))
}

pub fn try_encrypt(
    file_path: &Path,
    hash_from_key: [u8; 32],
) -> error::Result<()> {
    let target_file_path = &file_path.with_extension("enc");

    let mut rng = thread_rng();
    let rand_string = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(u8::from)
        .take(19)
        .collect::<Vec<u8>>();

    let nonce = array_ref![rand_string.as_slice(), 0, 19];
    {
        let mut source_file = File::open(file_path)?;
        let file_len = source_file.metadata()?.len() as usize;

        if target_file_path.exists() {
            return Err(error::Error::new_file_already_exist(
                target_file_path.to_str().unwrap_or(""),
            ));
        }

        {
            // Truncate the file
            let dist_file = File::open_or_create(target_file_path)?;
            dist_file.set_len(0)?;
        }

        let mut dist_file = File::open_or_create(target_file_path)?;

        let enc_meta = EncryptedMeta::new(
            file_path
                .file_name()
                .ok_or_else(|| {
                    error::Error::new_const(ErrorKind::OtherError, &"Internal")
                })?
                .to_str()
                .ok_or_else(|| {
                    error::Error::new_const(ErrorKind::OtherError, &"Internal")
                })?,
        );

        let raw_meta = RawMeta::new(&nonce, CipherKind::ChaCha20Poly1305);

        encrypt_file(
            &mut source_file,
            file_len,
            &mut dist_file,
            &hash_from_key,
            nonce,
            &enc_meta,
        )?;

        add_raw_meta(&raw_meta, &mut dist_file)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    #[test]
    fn get_hash() {
        assert_eq!(
            crate::get_hash("FM7348mwmw73t").unwrap(),
            hex!("66afe59af310865bc544c9d7a19ded0b1f8e6a1e797c3a1215a33175cae4023c")
        );
    }
}
