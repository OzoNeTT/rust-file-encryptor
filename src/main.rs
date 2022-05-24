mod file;
mod encryption;
mod meta;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::{io, iter, thread, time};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use file::{save_file, read_file};
use crate::encryption::{decrypt_file, encrypt_file};
use crate::file::OpenOrCreate;

use clap::{Arg, Parser};
use sha2::{Sha256};

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
) -> io::Result<&[u8; 32]> {

    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());

    let hashed_key = hasher.finalize();

    Ok(hashed_key)
}


fn main() -> io::Result<()> {
    let app_data = AppData::parse();
    println!("Filepath: {:?}", app_data.filepath);

    let path = std::env::args().nth(1).expect("No path specified!");

    let file_path = Path::new(&path);

    pretty_env_logger::init();

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
        encrypt_file(
            &file_path,
            &target_file_path,
            &key,
            nonce,
        )?;
    }

    println!("Go check");
    thread::sleep(time::Duration::from_secs(1));

    {
        //target_file.seek(SeekFrom::Start(MAGIC_STRING.len() as u64))?;

        decrypt_file(
            &target_file_path,
            &decrypt_file_path,
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
