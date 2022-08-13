use crate::meta::error::{ErrorKind as MetaErrorKind, MetaError};
use crate::meta_enc::EncryptedMeta;
use crate::{error, CipherKind};

const FILENAME: &'static str = "the filename";

fn fixtures() -> (Vec<u8>, EncryptedMeta) {
    (
        Vec::<u8>::new()
            .into_iter()
            .chain(FILENAME.bytes())
            .chain([0x00; 4])
            .collect(),
        EncryptedMeta {
            filename: FILENAME.try_into().unwrap(),
        },
    )
}

#[test]
fn test_from_vec_err_size() -> error::Result<()> {
    let result: Result<EncryptedMeta, MetaError> = vec![].try_into();
    assert!(result.is_err());

    let err = result.err().unwrap();
    assert_eq!(
        err.repr,
        MetaErrorKind::WrongEncryptedVecSize
    );

    Ok(())
}

#[test]
fn test_from_vec_err_string() -> error::Result<()> {
    let result: Result<EncryptedMeta, MetaError> = "asdasda"
        .bytes()
        .collect::<Vec<u8>>()
        .try_into();
    assert!(result.is_err());

    let err = result.err().unwrap();
    assert_eq!(
        err.repr,
        MetaErrorKind::WrongEncryptedWrongStringsAmount
    );

    Ok(())
}

#[test]
fn test_from_vec() -> error::Result<()> {
    let (vector, meta) = fixtures();

    let result: EncryptedMeta = vector.try_into()?;

    assert_eq!(result, meta);

    Ok(())
}
