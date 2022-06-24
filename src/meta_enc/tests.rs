use crate::error;
use crate::meta::{EncryptedMeta, META_MIN_SIZE};

const NONCE: [u8; 19] = [
    10u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 0u8, 1u8, 2u8, 3u8, 4u8,
    5u8, 6u8, 7u8, 8u8,
];

#[test]
fn to_vec() {
    let meta = EncryptedMeta::new(&NONCE, "file.txt");

    assert_eq!(
        meta.to_vec().as_slice(),
        concat!(
            "\x00file.txt\x00\x0A\x01\x02\x03\x04\x05\x06\x07",
            "\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08RFED"
        )
        .as_bytes(),
    );
}

#[test]
fn from_vec() -> error::Result<()> {
    let content: Vec<u8> = concat!(
        "\x00file.txt\x00\x0A\x01\x02\x03\x04\x05\x06\x07",
        "\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08RFED"
    )
    .as_bytes()
    .into();

    let result: EncryptedMeta = (&content).try_into()?;

    assert_eq!(
        result,
        EncryptedMeta::new(&NONCE, "file.txt"),
    );

    Ok(())
}

#[test]
fn test_is_valid_encoded() {
    let file: Vec<u8> = vec![2];
    assert_eq!(
        EncryptedMeta::is_valid_encoded(&file),
        false
    );

    let file: Vec<u8> = vec![1u8]
        .into_iter()
        .cycle()
        .take(META_MIN_SIZE)
        .chain(EncryptedMeta::MAGIC)
        .collect();
    assert_eq!(
        EncryptedMeta::is_valid_encoded(&file),
        true
    );
}
