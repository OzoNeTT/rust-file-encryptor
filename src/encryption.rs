use crate::error;
use crate::meta::error::MetaError;
use crate::meta::header::MetaHeader;
use crate::meta_raw::RawMeta;
use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn try_parse(source_file: &Path) -> error::Result<bool> {
    let mut file = File::open(source_file)?;
    let mut hdr_buff = vec![0u8; MetaHeader::size()];
    file.read_exact(&mut hdr_buff)?;

    match TryInto::<MetaHeader>::try_into(&hdr_buff).ok() {
        Some(meta) => Ok(meta.is_magic_valid()),
        None => Ok(false),
    }
}

pub fn get_raw_meta(file: &mut dyn Read) -> error::Result<RawMeta> {
    let mut hdr_buff = vec![0u8; MetaHeader::size()];
    file.read_exact(&mut hdr_buff)?;
    println!("hdr_buff: {:?}", hdr_buff);
    let header: MetaHeader = hdr_buff.try_into()?;

    let mut raw_buff = vec![0u8; header.size as usize];
    file.read_exact(&mut raw_buff)?;

    raw_buff
        .try_into()
        .map_err(|e: MetaError| e.into())
}

pub fn add_raw_meta(
    meta: &RawMeta,
    target_file: &mut dyn Write,
) -> error::Result<()> {
    let raw_meta_hdr = MetaHeader {
        size: meta.len() as u64,
        version: RawMeta::version(),
        magic: MetaHeader::MAGIC,
    };
    let raw_meta_hdr_vec = raw_meta_hdr.to_bytes();
    let raw_meta_vec = meta.to_bytes();

    target_file.write_all(&raw_meta_hdr_vec)?;
    target_file.write_all(&raw_meta_vec)?;

    Ok(())
}
