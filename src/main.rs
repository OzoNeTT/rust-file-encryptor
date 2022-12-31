use file_encryptor::app::context::{
    get_context_preview, user_key_hash, AppContext,
};
use file_encryptor::cli::args::get_arguments;
use file_encryptor::cli::runtime::command::register_all_commands;
use file_encryptor::cli::runtime::CommandProcessorContext;
use file_encryptor::encryption::try_parse;
use file_encryptor::{error, get_hash, try_decrypt, try_encrypt};
use path_absolutize::*;
use rpassword::prompt_password;
use std::fs::remove_file;
use std::path::Path;
use std::{env, io};

/// Log level is being controlled by the ENV variable RUST_LOG
///
/// Available log levels:
/// - error
/// - warn
/// - info
/// - debug
/// - trace
fn init_logger() {
    use log::LevelFilter;
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();
}

fn cli_mode(
    mut ctx: AppContext,
    mut cmd_context: CommandProcessorContext<AppContext>,
) -> error::Result<()> {
    while !ctx.cli_exit {
        let line = match cmd_context
            .lines_processing(&mut ctx, &mut console::Term::stdout())
        {
            Ok(v) => v,
            Err(e) => {
                log::error!(target: "app_main", "Error: {e}");
                continue;
            }
        };
        log::debug!(target: "app_main", "Got line: {line}");
    }

    Ok(())
}

fn main() -> error::Result<()> {
    init_logger();
    let mut cmd_context: CommandProcessorContext<AppContext> =
        CommandProcessorContext::new();
    let term = console::Term::stdout();
    let data = get_arguments(env::args_os());
    let mut ctx = AppContext {
        cli_current_path: Path::new(&data.filepath)
            .absolutize()?
            .to_path_buf(),
        cli_exit: false,
        key_hash: None,
        data,
        term,
    };
    register_all_commands(&mut cmd_context);

    ctrlc::set_handler(move || {
        // Do nothing
    })
    .map_err(|e| error::Error::new(error::ErrorKind::OtherError, e))?;
    log::info!(target: "app_main", "Use `exit` command to exit, `help` to get help\n");

    if ctx.data.key.is_some() {
        ctx.key_hash = Some(get_hash(
            ctx.data.key.unwrap().as_str(),
        )?);
        ctx.data.key = None;
    }

    if ctx.data.cli.unwrap_or(false) {
        return cli_mode(ctx, cmd_context);
    }

    let file_path = Path::new(&ctx.data.filepath).absolutize()?;
    log::info!(target: "app_main", "Filepath: {:?}", file_path);

    if !file_path.exists() {
        return Err(error::Error::new_file_not_found(
            file_path.to_str().unwrap_or(""),
        ));
    }
    log::debug!(target: "app_main", "File exists, ok");

    let mut preview: bool = false;
    if try_parse(file_path.as_ref())? {
        preview = get_context_preview(&ctx)?;
        log::debug!(target: "app_main","Preview arg is: {:?}", preview);

        let key = match &ctx.data.key {
            Some(key) => key.clone(),
            None => prompt_password("Enter the key: ")?,
        };

        let hash_from_key = get_hash(&key)?;
        log::debug!(target: "app_main", "Key entered");

        println!("Encrypted file will be decrypted");
        try_decrypt(
            file_path.as_ref(),
            hash_from_key,
            preview,
        )?;
    } else {
        println!("Raw file will be encrypted");
        log::debug!(target: "app_main", "Key entered");
        if ctx.key_hash.is_none() {
            ctx.key_hash = Some(user_key_hash()?);
        }

        // to encrypt
        try_encrypt(
            file_path.as_ref(),
            None,
            ctx.key_hash.unwrap(),
        )?;
    }

    // TODO: encapsulate
    if !ctx.data.keep_original && !preview {
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
