use crate::meta_enc::parser::DynamicParser;

const ARRAY_TO_PARSE: &[u8; 63] = b"\x08\x00a_large_file_name with (spaces) and [brackets].txt.fghfghfgh\0";

#[test]
fn parse_le_signelele_batche() {
    let mut parser = DynamicParser::new();
    parser.parse_next(ARRAY_TO_PARSE).expect("TODO: panic message");

    let result = parser.to_encrypted_meta().expect("Panic");
    assert_eq!(result.filename, "a_large_file_name with (spaces) and [brackets].txt.fghfghfghs");
}