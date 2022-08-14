use crate::meta::error::{ErrorKind as MetaErrorKind, MetaError};

#[cfg(test)]
mod tests;

pub const META_MAGIC_SIZE: usize = 3usize;
pub const META_HDR_RESERVED_SIZE: usize = 8 - 1 - META_MAGIC_SIZE;

/// Structure header
///
/// # Binary structure
///
/// - `MG` stands for Magic (bytes)
/// - `--` stands for Reserved
/// - `V` stands for Version
/// - `S` stands for Size (in Little Endian)
///
/// ```kotlin
///      0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
/// 0x00 MG MG MG V  -- -- -- -- S  S  S  S  S  S  S  S
/// ```
///
/// ## Example
///
/// Consider:
/// - the magic is `[0x01, 0x02, 0x03]`,
/// - version is `0xFF`
/// - and size is 0x5F3AD (LE encoding)
///
/// ```kotlin
///      0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
/// 0x00 01 02 03 FF 00 00 00 00 AD F3 05 00 00 00 00 00
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MetaHeader {
    /// Magic for determining
    pub magic: [u8; META_MAGIC_SIZE],

    /// Described struct version
    pub version: u8,

    /// Size of the described structure
    /// Used for looking up, how many bytes to read
    pub size: u64,
}

impl MetaHeader {
    pub const RESERVED: [u8; META_HDR_RESERVED_SIZE] =
        [0u8; META_HDR_RESERVED_SIZE];
    pub const MAGIC: [u8; META_MAGIC_SIZE] = [0x52, 0x46, 0x45];

    /// Length of the Header
    pub const fn len(&self) -> usize {
        Self::size()
    }

    pub const fn is_empty(&self) -> bool {
        false
    }
    /// Header object length is always constant
    pub const fn size() -> usize {
        0x10
    }

    pub fn is_magic_valid(&self) -> bool {
        self.magic == Self::MAGIC
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    pub fn to_bytes(&self) -> [u8; Self::size()] {
        let mut result: [u8; Self::size()] = [0u8; Self::size()];

        (&mut result[0..META_MAGIC_SIZE]).clone_from_slice(&self.magic);
        result[META_MAGIC_SIZE] = self.version;
        (&mut result[8..]).clone_from_slice(&self.size.to_le_bytes());

        log::trace!(target: "meta/header MetaHeader to_bytes", "Result: {result:?}");
        result
    }

    pub fn from_bytes(bytes: [u8; Self::size()]) -> Self {
        Self {
            magic: bytes[0..META_MAGIC_SIZE]
                .try_into()
                .expect("Wrong slice size"),
            version: bytes[META_MAGIC_SIZE],
            size: u64::from_le_bytes(
                bytes[8..]
                    .try_into()
                    .expect("Wrong slice size"),
            ),
        }
    }
}

impl TryFrom<&Vec<u8>> for MetaHeader {
    type Error = MetaError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        log::debug!(target: "meta/header MetaHeader try_from", "value.len(): {:?}",  value.len());

        Ok(Self::from_bytes(
            value
                .as_slice()
                .try_into()
                .map_err(|_| {
                    MetaError::from(MetaErrorKind::WrongHeaderVecSize)
                })?,
        ))
    }
}

impl TryFrom<Vec<u8>> for MetaHeader {
    type Error = MetaError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<[u8; MetaHeader::size()]> for MetaHeader {
    fn from(value: [u8; MetaHeader::size()]) -> Self {
        MetaHeader::from_bytes(value)
    }
}

impl From<&MetaHeader> for Vec<u8> {
    fn from(value: &MetaHeader) -> Self {
        value.to_vec()
    }
}

impl From<&MetaHeader> for [u8; MetaHeader::size()] {
    fn from(value: &MetaHeader) -> Self {
        value.to_bytes()
    }
}
