// use core::slice::SlicePattern;

use crate::meta_enc::EncryptedMeta;
use crate::meta_raw::RawMeta;
use crate::{error, OpenOrCreate};
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

    {
        let mut file = File::open(source_file)?;

        // TODO: buffered read the end of the file
        // (do not read all file)
        file.read_to_end(&mut buff)?;
        return match TryInto::<RawMeta>::try_into(&buff).ok() {
            Some(..) => Ok(true),
            None => Ok(false),
        };
    }
}

pub fn get_raw_meta(source_file: &Path) -> error::Result<RawMeta> {
    let mut buff = vec![];

    let mut file = File::open_read_only(source_file)?;
    file.read_to_end(&mut buff)?;

    println!("buffer: {buff:?}");

    (&buff).try_into()
}

pub fn add_raw_meta(
    meta: &RawMeta,
    target_file: &mut dyn Write,
) -> error::Result<()> {
    let raw_meta_vec = meta.to_vec();
    println!("raw_meta_slice: {:?}", raw_meta_vec);

    target_file.write_all(raw_meta_vec.as_slice())?;

    Ok(())
}
