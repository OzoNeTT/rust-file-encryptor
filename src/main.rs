mod file;
mod encryption;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::{io, iter};
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use file::{save_file, read_file};
use crate::encryption::{decrypt_file, encrypt_file};
use crate::file::OpenOrCreate;

#[macro_use]
extern crate log;
extern crate core;


fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let file_path = Path::new("./bin/sample.txt");
    let target_file_path = Path::new("./bin/sample.txt.enc");
    let decrypt_file_path = Path::new("./bin/sample.txt.dec");

    let key = [0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8];

    let mut rng = thread_rng();
    let rand_string = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(u8::from)
        .take(19)
        .collect::<Vec<u8>>();

    let nonce = rand_string.as_slice();

    {
        let mut source_file = File::open(file_path)?;
        let mut target_file = File::open_or_create(target_file_path)?;

        encrypt_file(
            &mut source_file,
            &mut target_file,
            &key,
            nonce,
        )?;
    }
    {
        let mut target_file = File::open(target_file_path)?;
        let mut target_dec_file = File::open_or_create(decrypt_file_path)?;
        decrypt_file(
            &mut target_file,
            &mut target_dec_file,
            &key,
            nonce,
        )?;
    }

    // log::warn!("Testing");
    //
    // for i in 0..content.len() {
    //     content[i] = 0x68;
    // }

    // save_file(content, file_path)?;


    // println!("{}");

    println!("Hello, pidor!");

    Ok(())
}
