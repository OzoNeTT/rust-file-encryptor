use clap::Parser;
use file_encryptor::encryption::try_parse;
use file_encryptor::{error, get_hash, try_decrypt, try_encrypt};
use rpassword::prompt_password;
use std::fs::remove_file;
use std::io;
use std::io::Write;
use std::path::Path;

// TODO: make mod app
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

    if preview {
        let mut buffer: String = String::new();
        io::stdin().read_line(&mut buffer)?;
    }

    Ok(())
}
