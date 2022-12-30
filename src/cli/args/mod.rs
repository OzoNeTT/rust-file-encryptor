use clap::Parser;
use std::ffi::OsString;

#[derive(Parser, Debug, Clone)]
pub struct AppData {
    #[clap(help = "Path to the file", required = true)]
    pub filepath: String,

    #[clap(short = 'k', long = "key", help = "Key")]
    pub key: Option<String>,

    #[clap(short = 'p', long = "preview", help = "Preview-only mode")]
    pub preview: Option<bool>,

    #[clap(short = 'c', long = "cli", help = "Runtime CLI mode")]
    pub cli: Option<bool>,

    #[clap(long = "keep", help = "Do not delete original file")]
    pub keep_original: bool,
}

pub fn get_arguments<I, T>(itr: I) -> AppData
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    AppData::parse_from(itr)
}
