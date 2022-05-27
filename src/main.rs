mod file;
mod encryption;
mod meta;
use arrayref::array_ref;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::{io, iter, thread, time};
use std::borrow::Borrow;
use std::convert::TryInto;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;


use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use file::{save_file, read_file};
use crate::encryption::{decrypt_file, try_parse, encrypt_file, append_meta, get_and_remove_meta};
use crate::file::OpenOrCreate;

use clap::{Arg, Parser};
use sha2::{Sha256, Digest};

#[macro_use]
extern crate log;
extern crate core;

#[derive(Parser, Debug)]
struct AppData {
    #[clap(help = "Path to the file")]
    pub filepath: String,

    #[clap(help = "Key", required=true)]
    pub key: Option<String>,
}

fn get_hash(
    key: &str,
) -> io::Result<[u8; 32]> {

    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());

    let hashed_key: [u8; 32] = hasher.finalize().as_slice().try_into().expect(
        "I don't know wtf?"
    );
    Ok(hashed_key)
}


fn main() -> io::Result<()> {
    let app_data = AppData::parse();
    println!("Filepath: {:?}", app_data.filepath);

    //TODO: validate if path correct

    let file_path = Path::new(&app_data.filepath);
    if !file_path.exists(){
        Err(io::Error::new(ErrorKind::Other, "Invalid filepath!"))?;
    }

    let key = match app_data.key {
        Some(key) => key,
        None => "".to_string(),
    };


    if key == "" {
        Err(io::Error::new(ErrorKind::Other, "No key provided!"))?;
    }

    let hash_from_key = get_hash(&key)?;

    pretty_env_logger::init();

    //let target_file_path = Path::new("./bin/sample.txt.enc");
    //let decrypt_file_path = Path::new("./bin/sample.txt.dec");
    let target_file_path = file_path;
    let decrypt_file_path = file_path;

    if try_parse(&file_path)? {
        //to decrypt
        {
            //target_file.seek(SeekFrom::Start(MAGIC_STRING.len() as u64))?;
            let meta = get_and_remove_meta(
                &target_file_path
            )?;
            let nonce = meta.nonce;
            let result = match decrypt_file(
                &target_file_path,
                &decrypt_file_path,
                &hash_from_key,
                &nonce,
            ) {
                Ok(result) => result,
                Err(_) => false
            };

            if !result{
                println!("Not correct key provided!")
            }

        }

    } else {
        // to encrypt
        let mut rng = thread_rng();
        let rand_string = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(u8::from)
            .take(19)
            .collect::<Vec<u8>>();

        let nonce = array_ref![rand_string.as_slice(), 0, 19];
        {
            encrypt_file(
                &file_path,
                &target_file_path,
                &hash_from_key,
                nonce,
            )?;

            append_meta(
                nonce,
                &target_file_path
            )?;
        }
    }

    println!("Job done!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    #[test]
    fn get_hash()  {
        assert_eq!(
            crate::get_hash("FM7348mwmw73t").unwrap(),
            hex!("66afe59af310865bc544c9d7a19ded0b1f8e6a1e797c3a1215a33175cae4023c")
        );
    }
}