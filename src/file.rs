use std::fs::{File, OpenOptions};
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

#[allow(dead_code)]
pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    {
        let mut file = File::open(path)?;

        file.read_to_end(&mut buffer)?;
    }
    Ok(buffer)
}

pub trait OpenOrCreate {
    fn open_or_create(path: &Path) -> io::Result<File>;
    fn open_write(path: &Path) -> io::Result<File>;
    fn open_append(path: &Path) -> io::Result<File>;
}

impl OpenOrCreate for File {
    fn open_or_create(path: &Path) -> io::Result<File> {
        if path.exists() {
            // TODO: needed to loop for encryption in dir
            if !path.is_file() {
                return Err(io::Error::new(ErrorKind::InvalidData, ""));
            }
        }

        if path.is_file() {
            println!("Opening the file {0:?}", path);

            return OpenOptions::new()
                .create_new(false)
                .write(true)
                .append(false)
                .open(path);
        }

        return OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(false)
            .open(path);
    }


    fn open_write(path: &Path) -> io::Result<File> {
        return OpenOptions::new()
            .write(true)
            .append(false)
            .open(path);
    }

    fn open_append(path: &Path) -> io::Result<File> {
        return OpenOptions::new()
            .write(true)
            .append(true)
            .open(path);
    }
}

#[allow(dead_code)]
pub fn save_file(data: Vec<u8>, path: &Path) -> io::Result<()> {
    return Ok(
        File::open_or_create(path)?
            .write_all(&data)?
    );
}
