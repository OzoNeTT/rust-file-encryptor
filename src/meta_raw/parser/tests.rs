use crate::meta_raw::parser::MetaRawDynamicParser;
use crate::utils::DynamicParser;
use crate::{error, CipherKind, ErrorKind, RawMeta};
use std::error::Error;

#[test]
fn parse_invalid_magic() -> error::Result<()> {
    const ARRAY: &[u8] =
        b"\x01\x01\x01\0x01\x01\x01\x01\0x01\x01\x01\x01\0x01\x01\x01\x01\0x01";

    let mut parser = MetaRawDynamicParser::new();
    let opt_err = parser.parse_next(&ARRAY).err();

    assert_eq!(opt_err.is_some(), true);

    let err = opt_err.unwrap();
    assert_eq!(
        err.kind(),
        ErrorKind::RawMetaDecodeError
    );
    assert!(err.to_string().contains("meta"));

    Ok(())
}

#[test]
fn parse_not_ready() {
    const ARRAY: &[u8] = b"\x52\x46\x45\x44\x65\x06";

    let mut parser = MetaRawDynamicParser::new();
    parser.parse_next(ARRAY.clone());

    let opt_err = parser.to_raw_meta().err();
    assert!(opt_err.is_some());

    let err = opt_err.unwrap();
    assert_eq!(err.kind(), ErrorKind::RawMetaIsNotReady);
}

#[test]
fn parse_wrong_cipher_type() {
    let array = [
        b"\x52\x46\x45\x44\xFF\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00"
            as &[u8],
        b"\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA"
            as &[u8],
        b"\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00"
            as &[u8],
    ]
    .concat();

    let mut parser = MetaRawDynamicParser::new();
    parser.parse_next(&array.as_slice());

    let opt_err = parser.to_raw_meta().err();
    assert!(opt_err.is_some());

    let err = opt_err.unwrap();
    assert_eq!(
        err.kind(),
        ErrorKind::FileInvalidCipherId
    );
    assert!(err.to_string().contains("255"));
}

#[test]
fn parse_success() -> error::Result<()> {
    let array = [
        b"\x52\x46\x45\x44\x01\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00"
            as &[u8],
        b"\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\xDE\xAD\xDE\xAD\xDE\xAD"
            as &[u8],
        b"\xDE\xAD\xDE\xAD\xDE\xAD\xDE\xAD" as &[u8],
    ]
    .concat();

    let mut parser = MetaRawDynamicParser::new();
    let tail = parser.parse_next(&array.as_slice())?;
    assert_eq!(
        tail,
        b"\xDE\xAD\xDE\xAD\xDE\xAD\xDE\xAD\xDE\xAD\xDE\xAD\xDE\xAD"
    );

    let result = parser.to_raw_meta()?;
    assert_eq!(result.cipher_kind, CipherKind::AesGcm);
    assert_eq!(result.magic, RawMeta::MAGIC);
    assert_eq!(&result.nonce, b"\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA");

    Ok(())
}

#[test]
fn parse_divided_batch() -> error::Result<()> {
    const ARRAY: [&[u8]; 5] = [
        b"\x52\x46",
        b"\x45\x44\x01",
        b"\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00",
        b"\xAA\x00\xAA\x00\xAA\x00\xAA",
        b"\x00\xAA\xDE\xAD\xDE\xAD\xDE\xAD",
    ];

    let mut parser = MetaRawDynamicParser::new();

    let mut tail = parser.parse_next(ARRAY[0])?;
    assert!(!parser.ready());
    assert_eq!(tail, []);

    let mut tail = parser.parse_next(ARRAY[1])?;
    assert!(!parser.ready());
    assert_eq!(tail, []);

    let mut tail = parser.parse_next(ARRAY[2])?;
    assert!(!parser.ready());
    assert_eq!(tail, []);

    let mut tail = parser.parse_next(ARRAY[3])?;
    assert!(!parser.ready());
    assert_eq!(tail, []);

    let mut tail = parser.parse_next(ARRAY[4])?;
    assert!(parser.ready());
    assert_eq!(tail, b"\xDE\xAD\xDE\xAD\xDE\xAD");

    let result = parser.to_raw_meta()?;
    assert_eq!(result.cipher_kind, CipherKind::AesGcm);
    assert_eq!(result.magic, RawMeta::MAGIC);
    assert_eq!(&result.nonce, b"\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA\x00\xAA");

    Ok(())
}
