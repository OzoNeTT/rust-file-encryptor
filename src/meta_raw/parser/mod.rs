use crate::error;
use crate::error::ErrorKind;
use crate::meta_enc::EncryptedMeta;
use crate::meta_raw::{RawMeta, MAGIC_SIZE, NONCE_SIZE};
use crate::utils::DynamicParser;
use std::cmp::min;

#[cfg(test)]
mod tests;

pub struct MetaRawDynamicParser {
    magic: Vec<u8>,
    cipher_kind: Option<u8>,
    nonce: Vec<u8>,

    struct_state: DynamicParserState,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum DynamicParserState {
    Empty,
    FilledMagic,
    FilledCipherKind,
    FullFilled,
}

impl DynamicParser for MetaRawDynamicParser {
    fn new() -> Self {
        Self {
            magic: Vec::new(),
            cipher_kind: None,
            nonce: Vec::new(),

            struct_state: DynamicParserState::Empty,
        }
    }

    fn parse_next<'a>(
        &mut self,
        batch: &'a [u8],
    ) -> crate::error::Result<&'a [u8]> {
        let batch_len = batch.len();
        if batch_len < 1 {
            return Err(
                error::Error::new(ErrorKind::RawMetaIsEmpty,
                                  format!("Expected meta batch to have at least 1 byte, got {batch_len}"))
            );
        }

        let mut batch_clipped: &[u8] = batch;
        while !batch_clipped.is_empty() {
            match self.struct_state {
                DynamicParserState::Empty => {
                    let position = min(
                        MAGIC_SIZE - self.magic.len(),
                        batch_clipped.len(),
                    );
                    let slice = &batch_clipped[..position];
                    self.magic.extend_from_slice(slice);

                    batch_clipped = &batch_clipped[position..];
                    if self.magic.len() == MAGIC_SIZE {
                        if self.magic != RawMeta::MAGIC {
                            return Err(error::Error::new_const(
                                ErrorKind::RawMetaDecodeError,
                                &"Raw meta invalid magic",
                            ));
                        }

                        self.struct_state = DynamicParserState::FilledMagic;
                    }
                }
                DynamicParserState::FilledMagic => {
                    self.cipher_kind = batch_clipped[0].into();

                    batch_clipped = &batch_clipped[1..];
                    self.struct_state = DynamicParserState::FilledCipherKind;
                }
                DynamicParserState::FilledCipherKind => {
                    let position = min(
                        NONCE_SIZE - self.nonce.len(),
                        batch_clipped.len(),
                    );
                    let slice = &batch_clipped[..position];
                    self.nonce.extend_from_slice(slice);

                    batch_clipped = &batch_clipped[position..];
                    if self.nonce.len() == NONCE_SIZE {
                        self.struct_state = DynamicParserState::FullFilled;
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

impl MetaRawDynamicParser {
    pub fn to_raw_meta(&self) -> error::Result<RawMeta> {
        if !self.ready() {
            return Err(ErrorKind::RawMetaIsNotReady.into());
        }

        Ok(RawMeta::new(
            <&[u8; 19]>::try_from(self.nonce.as_slice())?,
            self.cipher_kind
                .ok_or_else(|| {
                    error::Error::new_const(
                        ErrorKind::RawMetaDecodeError,
                        &"No cipher kind",
                    )
                })?
                .try_into()?,
        ))
    }
}

impl TryFrom<MetaRawDynamicParser> for RawMeta {
    type Error = error::Error;

    fn try_from(parser: MetaRawDynamicParser) -> Result<Self, Self::Error> {
        parser.to_raw_meta()
    }
}
