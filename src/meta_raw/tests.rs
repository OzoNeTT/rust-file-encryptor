use crate::meta::error::{ErrorKind as MetaErrorKind, MetaError};
use crate::meta_raw::{RawMeta, META_RAW_NONCE_SIZE};
use crate::{error, CipherKind};

const NONCE: [u8; META_RAW_NONCE_SIZE] = [
    10u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 0u8, 1u8, 2u8, 3u8, 4u8,
    5u8, 6u8, 7u8, 8u8,
];

const RAW_TEMPLATE: RawMeta = RawMeta {
    nonce: NONCE,
    cipher_kind: CipherKind::ChaCha20Poly1305,
};

fn fixtures() -> ([u8; RawMeta::size()]) {
    {
        let mut x = [0u8; RawMeta::size()];

        x[0] = CipherKind::ChaCha20Poly1305.to_u8();
        (&mut x[RawMeta::NONCE_BYTE_INDEX..]).clone_from_slice(&NONCE);

        x
    }
}

#[test]
fn test_from_bytes_err_cipherkind() -> error::Result<()> {
    let bytes = fixtures();

    let mut bytes = bytes.clone();
    bytes[0] = 0xFF;

    let result: Result<RawMeta, MetaError> = bytes.try_into();
    assert!(result.is_err());

    let err = result.err().unwrap();
    assert_eq!(
        err.repr,
        MetaErrorKind::WrongRawCipherKind
    );

    Ok(())
}

#[test]
fn test_from_bytes() -> error::Result<()> {
    let bytes = fixtures();
    let header: RawMeta = bytes.try_into()?;

    assert_eq!(RAW_TEMPLATE, header,);

    Ok(())
}

#[test]
fn test_to_bytes() -> error::Result<()> {
    let bytes = fixtures();

    assert_eq!(RAW_TEMPLATE.to_bytes(), bytes,);

    Ok(())
}

#[test]
fn test_from_vec_err() -> error::Result<()> {
    let bytes = fixtures();

    let result: Result<RawMeta, MetaError> =
        (&bytes[..RawMeta::size() - 1].to_vec()).try_into();
    assert!(result.is_err());

    let err = result.err().unwrap();
    assert_eq!(err.repr, MetaErrorKind::WrongRawVecSize);

    Ok(())
}

#[test]
fn test_from_vec() -> error::Result<()> {
    let bytes = fixtures();

    let result: RawMeta = (&bytes.to_vec()).try_into()?;

    assert_eq!(result, RAW_TEMPLATE);

    Ok(())
}
