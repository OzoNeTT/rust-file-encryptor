use crate::meta_enc::parser::MetaEncryptedDynamicParser;
use crate::utils::DynamicParser;
use crate::{error, ErrorKind};
use std::error::Error;
use std::io;
use std::io::Write;

#[test]
fn parse_single_batch() -> error::Result<()> {
    const ARRAY: &[u8] =
        b"\x3F\x00a_large_file_name with (spaces) and [brackets].txt.fghfghfgh\0";

    let mut parser = MetaEncryptedDynamicParser::new();
    let array = parser.parse_next(ARRAY)?;

    let result = parser.to_encrypted_meta()?;
    assert_eq!(
        result.filename,
        "a_large_file_name with (spaces) and [brackets].txt.fghfghfgh"
    );
    assert_eq!(array, []);

    Ok(())
}

#[test]
fn parse_divided_batch() -> error::Result<()> {
    const ARRAY: [&'static str; 5] = [
        "\x3F\x00a_l",
        "arge_file_name with ",
        "(spaces) and [brackets",
        "].txt.fghfghfgh",
        "\0tailed trailer",
    ];

    let mut parser = MetaEncryptedDynamicParser::new();

    let mut array = parser.parse_next(&ARRAY[0].as_bytes())?;
    assert_eq!(parser.ready(), false);
    assert_eq!(array, []);

    array = parser.parse_next(ARRAY[1].as_bytes())?;
    assert_eq!(parser.ready(), false);
    assert_eq!(array, []);

    array = parser.parse_next(ARRAY[2].as_bytes())?;
    assert_eq!(parser.ready(), false);
    assert_eq!(array, []);

    array = parser.parse_next(ARRAY[3].as_bytes())?;
    assert_eq!(parser.ready(), false);
    assert_eq!(array, []);

    array = parser.parse_next(ARRAY[4].as_bytes())?;
    assert_eq!(parser.ready(), true);
    assert_eq!(array, b"tailed trailer");

    let result = parser.to_encrypted_meta()?;
    assert_eq!(
        result.filename,
        "a_large_file_name with (spaces) and [brackets].txt.fghfghfgh"
    );

    Ok(())
}

#[test]
fn parse_size_overflow() -> error::Result<()> {
    const ARRAY: &[u8] = b"\x08\x00amongus\0asdasdasdasdads";

    let mut parser = MetaEncryptedDynamicParser::new();
    let opt_err = parser.parse_next(ARRAY).err();

    assert_eq!(opt_err.is_some(), true);
    let err = opt_err.unwrap();

    assert_eq!(
        err.kind(),
        ErrorKind::EncryptedMetaDecodeError
    );

    // TODO: message
    assert!(err.to_string().contains("overflow"));

    Ok(())
}

#[test]
fn parse_size_underflow() -> error::Result<()> {
    const ARRAY: &[u8] = b"\x08\x00gus\0asdasdasdasdads";

    let mut parser = MetaEncryptedDynamicParser::new();
    let opt_err = parser.parse_next(ARRAY).err();

    assert!(opt_err.is_some());
    let err = opt_err.unwrap();

    assert_eq!(
        err.kind(),
        ErrorKind::EncryptedMetaDecodeError
    );
    assert!(err.to_string().contains("underflow"));

    Ok(())
}
