use super::*;

#[test]
fn account_transaction_cursor_accepts_arbitrary_decimal_and_canonicalizes_zeroes() {
    for (value, expected) in [
        ("0", "0"),
        ("0000", "0"),
        ("42", "42"),
        ("00042", "42"),
        ("18446744073709551616", "18446744073709551616"),
        ("00018446744073709551616", "18446744073709551616"),
    ] {
        assert_eq!(normalize_transaction_cursor(value).unwrap(), expected);
    }
    for value in ["", "+1", "-1", " 1", "1 ", "1.0", "١", "４２"] {
        assert!(matches!(
            normalize_transaction_cursor(value),
            Err(IcrcAccountTransactionError::InvalidCursor { .. })
        ));
    }
}

#[test]
fn canonical_rows_require_unique_descending_arbitrary_size_ids() {
    let rows = |ids: &[&str]| {
        ids.iter()
            .map(|id| {
                serde_json::from_value::<IcrcAccountTransactionRow>(serde_json::json!({
                    "id": id,
                    "kind": "transfer",
                    "raw_transaction": {},
                }))
                .unwrap()
            })
            .collect::<Vec<_>>()
    };
    validate_canonical_account_transactions(&rows(&[
        "18446744073709551617",
        "18446744073709551616",
        "9",
        "0",
    ]))
    .expect("canonical newest-first rows beyond u64");
    for ids in [
        vec!["18446744073709551616", "18446744073709551616"],
        vec!["9", "10"],
        vec!["00042"],
        vec![""],
        vec!["+1"],
        vec!["-1"],
        vec!["01"],
        vec![" 1"],
        vec!["1 "],
        vec!["1.0"],
        vec!["١"],
        vec!["４２"],
    ] {
        assert!(validate_canonical_account_transactions(&rows(&ids)).is_err());
    }
}
