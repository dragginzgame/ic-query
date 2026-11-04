use super::*;
use serde_json::json;

#[test]
fn every_operation_preserves_decimal_strings_account_identifiers_and_null_fields() {
    for (operation, expected) in operation_cases() {
        let response = IcpIndexTransactionsResult::Ok(IcpIndexTransactions {
            balance: 0,
            transactions: vec![IcpIndexTransactionWithId {
                id: u64::MAX,
                transaction: IcpIndexTransaction {
                    memo: u64::MAX,
                    icrc1_memo: Some(vec![]),
                    operation,
                    created_at_time: Some(IcpIndexTimestamp {
                        timestamp_nanos: u64::MAX,
                    }),
                    timestamp: None,
                },
            }],
            oldest_tx_id: None,
        });
        let bytes = ic_platform::candid::contract::encode_one(&response).unwrap();
        let page = decode(&bytes, &Principal::management_canister())
            .unwrap()
            .unwrap();
        assert_eq!(page.transactions[0].id, "18446744073709551615");
        assert_eq!(
            page.transactions[0].raw_transaction,
            json!({
                "memo": "18446744073709551615", "icrc1_memo_hex": "",
                "operation": expected, "created_at_time_unix_nanos": "18446744073709551615",
                "timestamp_unix_nanos": null
            })
        );
    }
}

#[test]
fn deployed_icp_index_wire_shape_decodes_and_preserves_account_identifiers() {
    let response = IcpIndexTransactionsResult::Ok(IcpIndexTransactions {
        balance: 500,
        transactions: vec![IcpIndexTransactionWithId {
            id: 99,
            transaction: IcpIndexTransaction {
                memo: 7,
                icrc1_memo: Some(vec![0x01, 0x02]),
                operation: IcpIndexOperation::Transfer {
                    to: "to-account".to_string(),
                    fee: IcpIndexTokens { e8s: 10 },
                    from: "from-account".to_string(),
                    amount: IcpIndexTokens { e8s: 250 },
                    spender: Some("spender-account".to_string()),
                },
                created_at_time: Some(IcpIndexTimestamp {
                    timestamp_nanos: 1_700_000_000_000_000_000,
                }),
                timestamp: Some(IcpIndexTimestamp {
                    timestamp_nanos: 1_700_000_000_000_000_002,
                }),
            },
        }],
        oldest_tx_id: Some(7),
    });
    let bytes =
        ic_platform::candid::contract::encode_one(&response).expect("encode ICP index response");

    let page = decode(&bytes, &Principal::management_canister())
        .expect("decode ICP wire shape")
        .expect("successful ICP index response");
    let row = &page.transactions[0];

    assert_eq!(page.balance, "500");
    assert_eq!(page.oldest_transaction_id.as_deref(), Some("7"));
    assert_eq!(page.next_start(), Some("99"));
    assert_eq!(row.kind, "transfer");
    assert_eq!(row.amount_base_units.as_deref(), Some("250"));
    assert_eq!(row.fee_base_units.as_deref(), Some("10"));
    assert_eq!(
        row.from.as_ref().and_then(|account| match account {
            IcrcAccountRow::Legacy { account_identifier } => Some(account_identifier.as_str()),
            IcrcAccountRow::Structured { .. } => None,
        }),
        Some("from-account")
    );
    assert_eq!(row.memo_hex.as_deref(), Some("0102"));
    assert_eq!(row.raw_transaction["memo"], json!("7"));
    assert_eq!(
        row.raw_transaction["operation"]["spender_account_identifier"],
        json!("spender-account")
    );
}

fn operation_cases() -> [(IcpIndexOperation, serde_json::Value); 4] {
    [
        (
            IcpIndexOperation::Approve {
                fee: IcpIndexTokens { e8s: 0 },
                from: "from-account".to_string(),
                allowance: IcpIndexTokens { e8s: u64::MAX },
                expires_at: None,
                spender: "spender-account".to_string(),
                expected_allowance: None,
            },
            json!({
                "kind": "approve", "fee_base_units": "0",
                "from_account_identifier": "from-account",
                "allowance_base_units": "18446744073709551615",
                "expires_at_unix_nanos": null,
                "spender_account_identifier": "spender-account",
                "expected_allowance_base_units": null
            }),
        ),
        (
            IcpIndexOperation::Burn {
                from: "from-account".to_string(),
                amount: IcpIndexTokens { e8s: u64::MAX },
                spender: None,
            },
            json!({
                "kind": "burn", "from_account_identifier": "from-account",
                "amount_base_units": "18446744073709551615",
                "spender_account_identifier": null
            }),
        ),
        (
            IcpIndexOperation::Mint {
                to: "to-account".to_string(),
                amount: IcpIndexTokens { e8s: u64::MAX },
            },
            json!({
                "kind": "mint", "to_account_identifier": "to-account",
                "amount_base_units": "18446744073709551615"
            }),
        ),
        (
            IcpIndexOperation::Transfer {
                to: "to-account".to_string(),
                fee: IcpIndexTokens { e8s: u64::MAX },
                from: "from-account".to_string(),
                amount: IcpIndexTokens { e8s: 0 },
                spender: None,
            },
            json!({
                "kind": "transfer", "to_account_identifier": "to-account",
                "fee_base_units": "18446744073709551615",
                "from_account_identifier": "from-account", "amount_base_units": "0",
                "spender_account_identifier": null
            }),
        ),
    ]
}
