#[cfg(test)]
mod tests;

use crate::error;
use crate::error::ErrorKind;
use crate::meta_enc::EncryptedMeta;
use std::str::from_utf8;

pub struct DynamicParser {
    size: Option<u16>,
    filename: Vec<u8>,

    struct_state: DynamicParserState,
}

enum DynamicParserState {
    Empty,
    FilledSize,
    FullFilled,
}

impl DynamicParser {
    pub fn new() -> Self {
        Self {
            size: None,
            filename: Vec::new(),

            struct_state: DynamicParserState::Empty,
        }
    }

    pub fn parse_next<'a>(&mut self, batch: &'a [u8]) -> error::Result<&'a [u8]> {
        let batch_len = batch.len();
        if batch_len < 4 {
            return Err(
                error::Error::new(ErrorKind::EncryptedMetaIsEmpty,
                                  format!("Expected meta batch to have at least 4 bytes, got {batch_len}"))
            );
        }

        let mut batch_clipped: &[u8] = batch;

        while !batch_clipped.is_empty() {
            match self.struct_state {
                DynamicParserState::Empty => {
                    self.size = Some(batch[0] as u16 | ((batch[1] as u16) << 0x8));

                    batch_clipped = &batch[2..];
                    self.struct_state = DynamicParserState::FilledSize;
                }
                DynamicParserState::FilledSize => {
                    // TODO: consider the size :D

                    match batch_clipped.iter().position(|c| *c == 0x00) {
                        None => {
                            // No end symbol
                            self.filename.extend_from_slice(batch_clipped);
                        }
                        Some(index) => {
                            // Yesssss
                            self.filename.extend_from_slice(&batch_clipped[..index]);

                            batch_clipped = &batch_clipped[index + 1..];
                            self.struct_state = DynamicParserState::FullFilled;
                        }
                    }
                }
                DynamicParserState::FullFilled => {
                    break;
                }
            }
        }

        Ok(batch_clipped)
    }

    pub fn to_encrypted_meta(&self) -> error::Result<EncryptedMeta> {
        Ok(EncryptedMeta::new(from_utf8(
            self.filename.as_slice(),
        )?))
    }
}

impl TryFrom<DynamicParser> for EncryptedMeta {
    type Error = error::Error;

    fn try_from(parser: DynamicParser) -> Result<Self, Self::Error> {
        parser.to_encrypted_meta()
    }
}
