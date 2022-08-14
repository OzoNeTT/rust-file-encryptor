use clap::Parser;
use file_encryptor::encryption::try_parse;
use file_encryptor::{error, get_hash, try_decrypt, try_encrypt};
use path_absolutize::*;
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

/// Log level is being controlled by the ENV variable RUST_LOG
///
/// Available log levels:
/// - error
/// - warn
/// - info
/// - debug
/// - trace
fn init_logger() {
    pretty_env_logger::init();
}

fn main() -> error::Result<()> {
    init_logger();

    let app_data: AppData = AppData::parse();
    let file_path = Path::new(&app_data.filepath).absolutize()?;
    log::info!(target: "app_main", "Filepath: {:?}", file_path);

    if !file_path.exists() {
        return Err(error::Error::new_file_not_found(
            file_path.to_str().unwrap_or(""),
        ));
    }
    log::debug!(target: "app_main", "File exists, ok");

    let key = match app_data.key {
        Some(key) => key,
        None => prompt_password("Enter the key: ")?,
    };

    let hash_from_key = get_hash(&key)?;
    log::debug!(target: "app_main", "Key entered");

    //let target_file_path = Path::new("./bin/sample.txt.enc");
    //let decrypt_file_path = Path::new("./bin/sample.txt.dec");

    //let mut meta_info: EncryptedMeta;

    let mut preview: bool = false;
    if try_parse(file_path.as_ref())? {
        preview = match app_data.preview {
            Some(v) => {
                log::debug!(target: "app_main","Preview arg is defined: {:?}", v);
                v
            }
            None => {
                log::debug!(target: "app_main","Preview arg is undefined. Asking for preview");

                // TODO: encapsulate stdin somehow (macros maybe >.<)

                let mut dialog_result: Option<bool> = None;
                while dialog_result.is_none() {
                    print!("Preview the file content (do not create decrypted file) [Y/n]: ");
                    io::stdout().flush()?;

                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer)?;
                    buffer = buffer.trim().to_lowercase();

                    log::debug!("CLI read buffer: {:?}", buffer);
                    dialog_result = if buffer == "y" {
                        Some(true)
                    } else if buffer == "n" {
                        Some(false)
                    } else {
                        None
                    };
                }

                log::debug!(target: "app_main","Dialog result is: {:?}", dialog_result);
                dialog_result.unwrap()
            }
        };

        println!("Encrypted file will be decrypted");
        try_decrypt(
            file_path.as_ref(),
            hash_from_key,
            preview,
        )?;
    } else {
        println!("Raw file will be encrypted");

        // to encrypt
        try_encrypt(file_path.as_ref(), hash_from_key)?;
    }

    if !app_data.keep_original && !preview {
        remove_file(file_path.as_ref())?;
    }

    println!("Successfully!");

    if preview {
        log::debug!(target: "app_main", "Preview mode waiter");
        println!("Preview mode. Press [ENTER] to exit");

        let mut buffer: String = String::new();
        io::stdin().read_line(&mut buffer)?;
    }

    Ok(())
}
