pub mod encryption;
#[allow(dead_code)]
pub mod error;
pub mod file;
pub mod meta_raw;
pub mod meta_enc;
pub mod cipher;

use arrayref::array_ref;
use std::convert::TryInto;
use std::iter;
use std::path::Path;

use rand::{thread_rng, Rng};

use crate::encryption::{add_raw_meta, decrypt_file, encrypt_file, get_meta};
use crate::error::ErrorKind::WrongPassword;
use crate::file::OpenOrCreate;
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
    let meta = get_meta(file_path)?;
    let nonce = meta.nonce;

    //   .\\filename.txt
    //   ./source.\\filename
    //   ./source/.\\filename

    let filename = &meta.filename;
    //let decrypt_file_path = &file_path.to_owned()
    //    .with_file_name(filename);

    let decrypt_file_path = file_path.file_dir()?.join(filename);

    println!(
        "decrypt_file_path: {:?}",
        decrypt_file_path
    );
    decrypt_file(
        file_path,
        &decrypt_file_path,
        meta.len(),
        // 0,
        &hash_from_key,
        &nonce,
        preview,
    )?;

    Ok(())
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
        encrypt_file(
            file_path,
            target_file_path,
            &hash_from_key,
            nonce,
        )?;

        add_raw_meta(nonce, file_path, target_file_path)?;
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
