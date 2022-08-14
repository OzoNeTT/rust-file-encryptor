use crate::error;
use crate::meta::error::MetaError;
use crate::meta::header::MetaHeader;
use crate::meta::raw::RawMeta;
use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn try_parse(source_file: &Path) -> error::Result<bool> {
    let mut file = File::open(source_file)?;
    let file_len = file.metadata()?.len() as usize;

    log::debug!(target: "encryption try_parse", "File length: {file_len:?}");
    if file_len <= MetaHeader::size() {
        log::info!(target: "encryption try_parse", "File length is lower MetaHeader::size() = {:?}", MetaHeader::size());
        return Ok(false);
    }

    let mut hdr_buff = vec![0u8; MetaHeader::size()];
    file.read_exact(&mut hdr_buff)?;
    log::debug!(target: "encryption try_parse", "Read meta header into Vec<u8>");
    log::trace!(target: "encryption try_parse", "Meta header buffer: {hdr_buff:?}");

    match TryInto::<MetaHeader>::try_into(&hdr_buff).ok() {
        Some(meta) => {
            log::info!(target: "encryption try_parse", "File magic is valid");
            Ok(meta.is_magic_valid())
        }
        None => {
            log::info!(target: "encryption try_parse", "File magic is invalid");
            Ok(false)
        }
    }
}

pub fn get_raw_meta(file: &mut dyn Read) -> error::Result<RawMeta> {
    let mut hdr_buff = vec![0u8; MetaHeader::size()];
    file.read_exact(&mut hdr_buff)?;
    log::debug!(target: "encryption get_raw_meta", "Read meta header into Vec<u8>");
    log::trace!(target: "encryption get_raw_meta", "Meta header buffer: {hdr_buff:?}");

    let header: MetaHeader = hdr_buff.try_into()?;
    log::trace!(target: "encryption get_raw_meta", "MetaHeader: {header:?}");

    let mut raw_buff = vec![0u8; header.size as usize];
    file.read_exact(&mut raw_buff)?;
    log::debug!(target: "encryption get_raw_meta", "Read raw meta into Vec<u8>");
    log::trace!(target: "encryption get_raw_meta", "Raw meta buffer: {raw_buff:?}");

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
    log::trace!(target: "encryption add_raw_meta", "Meta header buffer: {raw_meta_hdr_vec:?}");
    log::trace!(target: "encryption add_raw_meta", "Raw meta buffer: {raw_meta_vec:?}");

    target_file.write_all(&raw_meta_hdr_vec)?;
    log::debug!(target: "encryption add_raw_meta", "Raw meta header written");

    target_file.write_all(&raw_meta_vec)?;
    log::debug!(target: "encryption add_raw_meta", "Raw meta written");

    Ok(())
}
