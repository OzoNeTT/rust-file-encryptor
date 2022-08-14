pub mod cipher;
pub mod encryption;
#[allow(dead_code)]
pub mod error;
pub mod file;
pub mod meta;
pub mod meta_enc;
pub mod meta_raw;

use arrayref::array_ref;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::{fs, io, iter};

use rand::{thread_rng, Rng};

use crate::cipher::kind::select_cipher;
use crate::cipher::CipherKind;
use crate::encryption::{add_raw_meta, get_raw_meta};
use crate::error::ErrorKind;
use crate::file::OpenOrCreate;
use crate::meta::header::MetaHeader;
use crate::meta_enc::EncryptedMeta;
use crate::meta_raw::RawMeta;
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
    let (meta, target_path) = {
        let target_file_path = &file_path.with_extension(
            file_path
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap_or("")
                .to_string()
                + ".tmp-enc",
        );

        let mut target: Box<dyn Write> = if preview {
            println!("\n----------------- [ cut here ] -----------------");
            Box::from(io::stdout()) as Box<dyn Write>
        } else {
            let mut file = File::open_or_create(target_file_path)?;

            Box::from(file) as Box<dyn Write>
        };

        let mut source = File::open_read_only(file_path)?;
        let raw_meta = get_raw_meta(&mut source)?;

        let file_len = source.metadata()?.len() as usize;
        let cipher = select_cipher(
            raw_meta.cipher_kind,
            Some(file_len - raw_meta.len()),
        );

        let enc_meta = cipher.decrypt(
            Box::from(source),
            target,
            &hash_from_key,
            &raw_meta.nonce,
        )?;
        if preview {
            println!("\n------------ [ end of the content ] ------------\n");
        }

        (enc_meta, target_file_path.clone())
    };

    if !preview {
        let real_target_path = file_path.with_file_name(meta.filename);
        println!(
            "Target {:?}, real target {:?}",
            target_path, real_target_path
        );
        fs::rename(target_path, real_target_path)?;
    }

    // Err(error::Error::new_const(
    //     ErrorKind::OtherError,
    //     &"Not implemented",
    // ))

    Ok(())
}

pub fn try_encrypt(
    file_path: &Path,
    hash_from_key: [u8; 32],
) -> error::Result<()> {
    let target_file_path = &file_path.with_extension("enc");

    println!("Target file path: {target_file_path:?}");

    let mut rng = thread_rng();
    let rand_string = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(u8::from)
        .take(19)
        .collect::<Vec<u8>>();

    let nonce = array_ref![rand_string.as_slice(), 0, 19];
    log::debug!(target: "lib try_encrypt", "Generated nonce");
    log::trace!(target: "lib try_encrypt", "Nonce: {nonce:?}");

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

        let filename = file_path
            .file_name()
            .ok_or_else(|| {
                error::Error::new_const(ErrorKind::OtherError, &"Internal")
            })?
            .to_str()
            .ok_or_else(|| {
                error::Error::new_const(ErrorKind::OtherError, &"Internal")
            })?;
        let enc_meta = EncryptedMeta {
            filename: filename.to_string(),
        };

        let raw_meta = RawMeta {
            cipher_kind: CipherKind::ChaCha20Poly1305,
            nonce: nonce.clone(),
        };
        add_raw_meta(&raw_meta, &mut dist_file)?;

        // println!("File len: {}, raw meta length: {}", file_len, raw_meta.len());
        let cipher = select_cipher(raw_meta.cipher_kind, Some(file_len));

        cipher.encrypt(
            Box::new(source_file),
            Box::new(dist_file),
            &hash_from_key,
            nonce,
            &enc_meta,
        )?;
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
