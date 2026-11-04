use super::*;

#[test]
fn malformed_subaccounts_return_typed_hex_errors() {
    for value in [
        format!("aé{}", "0".repeat(61)),
        "🦀".repeat(16),
        "+a".repeat(32),
        "gg".repeat(32),
        "a".repeat(63),
    ] {
        assert!(
            matches!(
                normalize_subaccount_hex(&value),
                Err(IcrcError::InvalidSubaccountHex { .. })
            ),
            "malformed subaccount {value:?}",
        );
        assert!(matches!(
            subaccount_bytes_from_hex(&value),
            Err(IcrcError::InvalidSubaccountHex { .. })
        ));
    }
}

#[test]
fn subaccounts_normalize_case_and_whitespace_and_require_32_bytes() {
    let mixed_case = "aB01".repeat(16);
    assert_eq!(
        normalize_subaccount_hex(&format!(" \t{mixed_case}\n")).expect("valid subaccount"),
        "ab01".repeat(16),
    );
    assert_eq!(
        subaccount_bytes_from_hex(&format!(" \t{mixed_case}\n"))
            .expect("wire bytes")
            .as_slice(),
        [0xab, 0x01].repeat(16),
    );
    for bytes in [0, 31, 33] {
        assert!(matches!(
            normalize_subaccount_hex(&"ab".repeat(bytes)),
            Err(IcrcError::InvalidSubaccountLength { bytes: actual }) if actual == bytes,
        ));
        assert!(matches!(
            subaccount_bytes_from_hex(&"ab".repeat(bytes)),
            Err(IcrcError::InvalidSubaccountLength { bytes: actual }) if actual == bytes,
        ));
    }
}
