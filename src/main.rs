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

#[macro_use]
extern crate log;
extern crate core;

fn main() -> io::Result<()> {
    let path = std::env::args().nth(1).expect("No path specified");

    let file_path = Path::new(&path);


    pretty_env_logger::init();

    let target_file_path = Path::new("./bin/sample.txt.enc");
    let decrypt_file_path = Path::new("./bin/sample.txt.dec");

    let key = [0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8,0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8, 0u8, 3u8, 4u8, 7u8];

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

    println!("Go check");
    thread::sleep(time::Duration::from_secs(4));

    {
        let mut target_file = File::open(target_file_path)?;
        let mut target_dec_file = File::open_or_create(decrypt_file_path)?;

        //target_file.seek(SeekFrom::Start(MAGIC_STRING.len() as u64))?;

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
