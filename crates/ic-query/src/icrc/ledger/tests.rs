use super::*;
use ic_platform::candid::contract::Int;

#[test]
fn native_index_and_standard_replies_preserve_external_shapes() {
    // Independent fixtures: didc encode of native Ok/Err variants and standard records.
    for (encoded, expected) in [
        (
            "4449444c016b01bc8a01680100000100",
            Ok(Principal::management_canister()),
        ),
        (
            "4449444c026b01c5fed201016b01cbfde8ed047f01000000",
            Err(GetIndexPrincipalError::IndexPrincipalNotSet),
        ),
        (
            "4449444c036b01c5fed201016b01d1c4987c026c02fc91f4f80571c498b1b50d7d0100000007666978747572652a",
            Err(GetIndexPrincipalError::GenericError {
                error_code: Nat::from(42_u8),
                description: "fixture".to_string(),
            }),
        ),
    ] {
        let bytes = crate::hex::decode_hex(encoded).unwrap();
        let actual: Result<Principal, GetIndexPrincipalError> =
            crate::candid_decode::decode_reply(&bytes).unwrap();
        assert_eq!(actual, expected);
    }
    let bytes = crate::hex::decode_hex("4449444c026d016c02efd6e40271cbe4fdc704710100011c68747470733a2f2f6578616d706c652e636f6d2f7374616e6461726406494352432d31").unwrap();
    let standards: Vec<SupportedStandard> = crate::candid_decode::decode_reply(&bytes).unwrap();
    let standards: Vec<_> = standards.into_iter().map(supported_standard_row).collect();
    assert_eq!(
        standards,
        vec![IcrcTokenStandardRow {
            name: "ICRC-1".to_string(),
            url: "https://example.com/standard".to_string()
        }]
    );
}

#[test]
fn metadata_conversion_preserves_native_kind_and_value() {
    for (raw, kind, value) in [
        (
            MetadataValue::Nat(Nat::from(7_u8)),
            IcrcMetadataValueKind::Nat,
            "7",
        ),
        (
            MetadataValue::Int(Int::from(-2_i8)),
            IcrcMetadataValueKind::Int,
            "-2",
        ),
        (
            MetadataValue::Nat("18446744073709551616".parse().unwrap()),
            IcrcMetadataValueKind::Nat,
            "18446744073709551616",
        ),
        (
            MetadataValue::Int("-18446744073709551616".parse().unwrap()),
            IcrcMetadataValueKind::Int,
            "-18446744073709551616",
        ),
        (
            MetadataValue::Text("token".to_string()),
            IcrcMetadataValueKind::Text,
            "token",
        ),
        (
            MetadataValue::Blob(vec![0x01, 0xab].into()),
            IcrcMetadataValueKind::Blob,
            "01ab",
        ),
    ] {
        let row = metadata_row("key".to_string(), raw);

        assert_eq!(row.key, "key");
        assert_eq!(row.value_type, kind);
        assert_eq!(row.value, JsonValue::String(value.to_string()));
    }
}
