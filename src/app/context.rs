use crate::cli::args::AppData;
use crate::cli::runtime::confirm::UserConfirm;
use crate::{error, get_hash};
use rpassword::prompt_password;
use std::path::PathBuf;

pub type KeyHashType = [u8; 32];

#[derive(Debug, Clone)]
pub struct AppContext {
    pub cli_current_path: PathBuf,
    pub cli_exit: bool,
    pub key_hash: Option<KeyHashType>,

    pub data: AppData,
    pub term: console::Term,
}

pub fn user_key_hash() -> error::Result<KeyHashType> {
    let key = loop {
        let password1 = prompt_password("Enter the key : ")?;
        let password2 = prompt_password("Repeat the key: ")?;

        if password1 != password2 {
            println!("Password does not match each other");
            continue;
        }

        break password1;
    };

    get_hash(&key)
}

pub fn set_context_key_hash(ctx: &mut AppContext) -> error::Result<()> {
    ctx.key_hash = Some(user_key_hash()?);
    Ok(())
}

pub fn get_context_preview(ctx: &AppContext) -> error::Result<bool> {
    let val = match ctx.data.preview {
        Some(v) => v,
        None => {
            log::debug!(target: "app_main","Preview arg is undefined. Asking for preview");
            ctx.term.clone().user_confirm_default(
                "Preview the file content (do not create decrypted file)",
                true,
            )?
        }
    };
    Ok(val)
}
