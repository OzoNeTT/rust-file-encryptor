use crate::meta::error::MetaError;
use crate::meta::header::MetaHeader;

const HEADER_TEMPLATE: MetaHeader = MetaHeader {
    magic: MetaHeader::MAGIC,
    size: 0x5f3ad,
    version: 0x3A,
};

const BYTES_TEMPLATE: [u8; 16] = [
    0x52, 0x46, 0x45, 0x3A, 0x00, 0x00, 0x00, 0x00, 0xAD, 0xF3, 0x05, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

#[test]
pub fn test_to_bytes() -> Result<(), MetaError> {
    let header = HEADER_TEMPLATE;

    assert!(header.is_magic_valid());

    let bytes = header.to_bytes();
    assert_eq!(bytes, BYTES_TEMPLATE);

    Ok(())
}

#[test]
pub fn test_from_bytes() -> Result<(), MetaError> {
    let bytes: [u8; MetaHeader::size()] = BYTES_TEMPLATE;

    let header: MetaHeader = bytes.into();
    assert_eq!(header, HEADER_TEMPLATE);

    Ok(())
}

#[test]
pub fn test_from_vec_error() -> Result<(), MetaError> {
    let vector: Vec<u8> = vec![];

    let result: Result<MetaHeader, MetaError> = (&vector).try_into();
    assert!(result.is_err());

    Ok(())
}

#[test]
pub fn test_from_vec() -> Result<(), MetaError> {
    let header: MetaHeader = (&BYTES_TEMPLATE.to_vec()).try_into()?;
    assert_eq!(header, HEADER_TEMPLATE);

    Ok(())
}
