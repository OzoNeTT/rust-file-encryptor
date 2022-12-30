use assert_fs::prelude::*;
use file_encryptor;
use file_encryptor::error;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, io};

const ROOT_FILE_DIR: &'static str = "tests/general/";
const TEMP_FILE_DIR: &'static str = "target/tmp";

fn setup() -> io::Result<()> {
    let temp_dir_path = PathBuf::from(".").join(TEMP_FILE_DIR);
    fs::create_dir_all(temp_dir_path.as_path())?;

    Ok(())
}

// TODO: remove copypasted tests

#[test]
fn test_common() -> error::Result<()> {
    setup()?;

    let temp = assert_fs::TempDir::new()?;
    temp.copy_from(ROOT_FILE_DIR, &["*.txt"])
        .expect("");

    let raw_file = temp.child("to_enc.txt");
    let key_hash = file_encryptor::get_hash("amongus").expect("");
    file_encryptor::try_encrypt((&raw_file).path(), key_hash.clone())?;

    fs::remove_file(raw_file.path()).expect("");

    let enc_file = temp.child("to_enc.enc");
    let enc_file_path = enc_file.path();
    println!("enc_file_path {enc_file_path:?}");

    fs::copy(
        enc_file_path,
        PathBuf::from(TEMP_FILE_DIR).join("enc.bin"),
    )?;

    file_encryptor::try_decrypt(enc_file_path, key_hash.clone(), false)?;

    let mut buffer = Vec::<u8>::with_capacity(512);
    File::open(
        Path::new(".")
            .join(ROOT_FILE_DIR)
            .join("to_enc.txt"),
    )
    .expect("")
    .read_to_end(&mut buffer)
    .expect("");

    let mut expected_buff = Vec::<u8>::with_capacity(512);
    File::open(temp.child("to_enc.txt").path())
        .expect("")
        .read_to_end(&mut expected_buff)
        .expect("");

    assert_eq!(buffer, expected_buff);

    // temp.close()?;

    Ok(())
}

#[test]
fn test_common_large() -> error::Result<()> {
    setup()?;

    let temp = assert_fs::TempDir::new()?;
    temp.copy_from(ROOT_FILE_DIR, &["*.txt"])
        .expect("");

    let raw_file = temp.child("to_enc_large.txt");
    let key_hash = file_encryptor::get_hash("amongus").expect("");
    file_encryptor::try_encrypt((&raw_file).path(), key_hash.clone())?;

    fs::remove_file(raw_file.path()).expect("");

    let enc_file = temp.child("to_enc_large.enc");
    let enc_file_path = enc_file.path();
    println!("enc_file_path {enc_file_path:?}");

    fs::copy(
        enc_file_path,
        PathBuf::from(TEMP_FILE_DIR).join("enc_large.bin"),
    )?;

    file_encryptor::try_decrypt(enc_file_path, key_hash.clone(), false)?;

    let mut buffer = Vec::<u8>::with_capacity(512);
    File::open(
        Path::new(".")
            .join(ROOT_FILE_DIR)
            .join("to_enc_large.txt"),
    )
    .expect("")
    .read_to_end(&mut buffer)
    .expect("");

    let mut expected_buff = Vec::<u8>::with_capacity(512);
    File::open(temp.child("to_enc_large.txt").path())
        .expect("")
        .read_to_end(&mut expected_buff)
        .expect("");

    assert_eq!(buffer, expected_buff);

    // temp.close()?;

    Ok(())
}

#[test]
fn test_common_small() -> error::Result<()> {
    setup()?;

    let temp = assert_fs::TempDir::new()?;
    temp.copy_from(ROOT_FILE_DIR, &["*.txt"])
        .expect("");

    let raw_file = temp.child("to_enc_small.txt");
    let key_hash = file_encryptor::get_hash("amongus").expect("");
    file_encryptor::try_encrypt((&raw_file).path(), key_hash.clone())?;

    fs::remove_file(raw_file.path()).expect("");

    let enc_file = temp.child("to_enc_small.enc");
    let enc_file_path = enc_file.path();
    println!("enc_file_path {enc_file_path:?}");

    fs::copy(
        enc_file_path,
        PathBuf::from(TEMP_FILE_DIR).join("enc_small.bin"),
    )?;

    file_encryptor::try_decrypt(enc_file_path, key_hash.clone(), false)?;

    let mut buffer = Vec::<u8>::with_capacity(512);
    File::open(
        Path::new(".")
            .join(ROOT_FILE_DIR)
            .join("to_enc_small.txt"),
    )
    .expect("")
    .read_to_end(&mut buffer)
    .expect("");

    let mut expected_buff = Vec::<u8>::with_capacity(512);
    File::open(temp.child("to_enc_small.txt").path())
        .expect("")
        .read_to_end(&mut expected_buff)
        .expect("");

    assert_eq!(buffer, expected_buff);

    // temp.close()?;

    Ok(())
}
