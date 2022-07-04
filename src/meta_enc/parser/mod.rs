#[cfg(test)]
mod tests;

use crate::error;
use crate::error::ErrorKind;
use crate::meta_enc::EncryptedMeta;
use crate::utils::DynamicParser;
use crate::ErrorKind::EncryptedMetaDecodeError;
use std::cmp::min;
use std::str::from_utf8;

pub struct MetaEncryptedDynamicParser {
    size: Option<u16>,
    filename: Vec<u8>,

    filled_size: u16,
    struct_state: DynamicParserState,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum DynamicParserState {
    Empty,
    FilledSize,
    FullFilled,
}

impl DynamicParser for MetaEncryptedDynamicParser {
    fn new() -> Self {
        Self {
            size: None,
            filename: Vec::new(),

            filled_size: 0,
            struct_state: DynamicParserState::Empty,
        }
    }

    fn parse_next<'a>(&mut self, batch: &'a [u8]) -> error::Result<&'a [u8]> {
        let batch_len = batch.len();
        if batch_len < 4 {
            return Err(
                error::Error::new(ErrorKind::EncryptedMetaIsEmpty,
                                  format!("Expected meta batch to have at least 4 bytes, got {batch_len}"))
            );
        }

        let mut batch_clipped: &[u8] = batch;

        while !batch_clipped.is_empty() {
            println!(
                "Batch clipped size: {}",
                batch_clipped.len()
            );
            match self.struct_state {
                DynamicParserState::Empty => {
                    self.size =
                        Some(batch[0] as u16 | ((batch[1] as u16) << 0x8));

                    batch_clipped = &batch[2..];
                    self.struct_state = DynamicParserState::FilledSize;
                }
                DynamicParserState::FilledSize => {
                    // TODO: consider the size :D

                    match batch_clipped
                        .iter()
                        .position(|c| *c == 0x00)
                    {
                        None => {
                            let size = self.size.unwrap_or(0xFFFF);
                            let delta_size = min(
                                size - self.filled_size,
                                batch_clipped.len() as u16,
                            );

                            // No end symbol
                            self.filename.extend_from_slice(
                                &batch_clipped[..(delta_size as usize)],
                            );

                            self.filled_size += delta_size;
                            batch_clipped =
                                &batch_clipped[(delta_size as usize)..];

                            if self.filled_size == size {
                                // Full size and no term char
                                return Err(
                                    error::Error::new_const(
                                        ErrorKind::EncryptedMetaDecodeError,
                                        &"No terminator character at the end of the string. Maybe size overflow",
                                    )
                                );
                            } else if self.filled_size > size {
                                return Err(
                                    error::Error::new_encrypted_meta_size_mismatch(
                                        size,
                                        self.filled_size,
                                    )
                                );
                            }
                        }
                        Some(index) => {
                            let size = self.size.unwrap_or(0xFFFF);

                            // Yesssss
                            self.filename
                                .extend_from_slice(&batch_clipped[..index]);

                            batch_clipped = &batch_clipped[index + 1..];
                            self.filled_size += (index as u16) + 1;

                            self.struct_state = DynamicParserState::FullFilled;
                            if self.filled_size != size {
                                return Err(
                                    error::Error::new_encrypted_meta_size_mismatch(
                                        size,
                                        self.filled_size,
                                    )
                                );
                            }
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

    fn ready(&self) -> bool {
        self.struct_state == DynamicParserState::FullFilled
    }
}

impl MetaEncryptedDynamicParser {
    pub fn to_encrypted_meta(&self) -> error::Result<EncryptedMeta> {
        if !self.ready() {
            return Err(ErrorKind::EncryptedMetaIsEmpty.into());
        }

        Ok(EncryptedMeta::new(from_utf8(
            self.filename.as_slice(),
        )?))
    }
}

impl TryFrom<MetaEncryptedDynamicParser> for EncryptedMeta {
    type Error = error::Error;

    fn try_from(
        parser: MetaEncryptedDynamicParser,
    ) -> Result<Self, Self::Error> {
        parser.to_encrypted_meta()
    }
}
