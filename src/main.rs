mod encryption;
#[allow(dead_code)]
mod error;
mod file;
mod meta;

use arrayref::array_ref;
use std::convert::TryInto;
use std::fs::remove_file;
use std::io::Write;
use std::path::Path;
use std::{io, iter};

use rpassword::prompt_password;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::encryption::{
    append_meta, decrypt_file, encrypt_file, get_meta, try_parse,
};
use crate::error::ErrorKind::WrongPassword;
use crate::file::OpenOrCreate;
use clap::Parser;
use file::GetFileDirectory;
use sha2::{Digest, Sha256};

extern crate core;
extern crate log;

#[derive(Parser, Debug)]
struct AppData {
    #[clap(help = "Path to the file", required = true)]
    pub filepath: String,

    #[clap(short = 'k', long = "key", help = "Key")]
    pub key: Option<String>,

    #[clap(short = 'p', long = "preview", help = "Preview-only mode")]
    pub preview: Option<bool>,

    #[clap(long = "keep", help = "Do not delete original file")]
    pub keep_original: bool,
}

fn get_hash(key: &str) -> error::Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());

    let hashed_key: [u8; 32] = hasher
        .finalize()
        .as_slice()
        .try_into()?;
    Ok(hashed_key)
}

fn try_decrypt(
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
    )
    .map_err(|e| error::Error::new(WrongPassword, e))?;

    Ok(())
}

fn try_encrypt(file_path: &Path, hash_from_key: [u8; 32]) -> error::Result<()> {
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
        )
        .map_err(|e| error::Error::new(WrongPassword, e))?;

        append_meta(nonce, file_path, target_file_path)?;
    }
    Ok(())
}

fn main() -> error::Result<()> {
    let app_data: AppData = AppData::parse();
    println!("Filepath: {:?}", app_data.filepath);

    //TODO: validate if path correct

    let file_path = Path::new(&app_data.filepath);
    if !file_path.exists() {
        return Err(error::Error::new_file_not_found(
            file_path.to_str().unwrap_or(""),
        ));
    }

    let key = match app_data.key {
        Some(key) => key,
        None => prompt_password("Enter the key: ")?,
    };

    println!("Key '{}' will be used", &key);

    let hash_from_key = get_hash(&key)?;

    pretty_env_logger::init();

    //let target_file_path = Path::new("./bin/sample.txt.enc");
    //let decrypt_file_path = Path::new("./bin/sample.txt.dec");

    //let mut meta_info: EncryptedMeta;

    let mut preview: bool = false;
    if try_parse(file_path)? {
        preview = match app_data.preview {
            Some(v) => v,
            None => {
                // TODO: encapsulate stdin somehow (macros maybe >.<)

                let mut dialog_result: Option<bool> = None;
                while dialog_result.is_none() {
                    print!("Preview the file content (do not create decrypted file) [Y/n]: ");
                    io::stdout().flush()?;

                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer)?;
                    buffer = buffer.trim().to_lowercase();
                    println!("buffer: {buffer:?}");

                    dialog_result = if buffer == "y" {
                        Some(true)
                    } else if buffer == "n" {
                        Some(false)
                    } else {
                        None
                    };
                }

                dialog_result.unwrap()
            }
        };

        try_decrypt(file_path, hash_from_key, preview)?;
    } else {
        // to encrypt
        try_encrypt(file_path, hash_from_key)?;
    }

    if !app_data.keep_original && !preview {
        remove_file(file_path)?;
    }

    println!("Job done!");

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
