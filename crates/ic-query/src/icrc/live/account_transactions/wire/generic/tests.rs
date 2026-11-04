use super::*;
use serde_json::{Value as JsonValue, json};

#[test]
fn every_operation_preserves_exact_decimal_blob_and_nullable_json_fields() {
    let response = IcrcIndexTransactionsResult::Ok(every_operation_transactions());
    let bytes = ic_platform::candid::contract::encode_one(&response).unwrap();
    let page = decode(&bytes, &Principal::management_canister())
        .unwrap()
        .unwrap();
    let from = json!({"owner": "2vxsx-fae", "subaccount_hex": "ff".repeat(32)});
    let to = json!({"owner": "aaaaa-aa", "subaccount_hex": null});
    let amount = "3402823669209384634633746074317682114561234";
    let timestamp = "18446744073709551615";
    assert_eq!(page.balance, amount);
    assert_eq!(page.transactions[0].id, amount);
    assert_eq!(
        page.transactions[0].amount_base_units.as_deref(),
        Some(amount)
    );
    assert_eq!(
        page.transactions[0].raw_transaction,
        json!({
            "kind": "transfer",
            "timestamp_unix_nanos": timestamp,
            "approve": {
                "fee_base_units": null, "from": from, "memo_hex": "",
                "created_at_time_unix_nanos": timestamp, "amount_base_units": amount,
                "expected_allowance_base_units": amount, "expires_at_unix_nanos": null,
                "spender": to
            },
            "burn": {
                "from": from, "memo_hex": null, "created_at_time_unix_nanos": null,
                "amount_base_units": amount, "spender": null, "fee_base_units": "0"
            },
            "mint": {
                "to": to, "memo_hex": "ff", "created_at_time_unix_nanos": null,
                "amount_base_units": amount, "fee_base_units": null
            },
            "transfer": {
                "to": to, "fee_base_units": amount, "from": from, "memo_hex": "00ff",
                "created_at_time_unix_nanos": null, "amount_base_units": amount, "spender": to
            },
            "fee_collector": {
                "caller": "2vxsx-fae", "fee_collector": to,
                "timestamp_unix_nanos": timestamp, "method": null
            },
            "authorized_mint": {
                "to": to, "amount_base_units": amount, "created_at_time_unix_nanos": null,
                "caller": null, "method": "mint", "reason": "retained evidence"
            },
            "authorized_burn": {
                "from": from, "amount_base_units": amount,
                "created_at_time_unix_nanos": timestamp, "caller": "aaaaa-aa",
                "method": null, "reason": null
            }
        })
    );
}

#[test]
fn current_index_wire_shape_round_trips_and_projects_approve_losslessly() {
    let owner = LedgerAccount {
        owner: Principal::anonymous(),
        subaccount: Some([0xaa; 32]),
    };
    let spender = LedgerAccount {
        owner: Principal::management_canister(),
        subaccount: None,
    };
    let response = IcrcIndexTransactionsResult::Ok(IcrcIndexTransactions {
        balance: Nat::from(500_u64),
        transactions: vec![IcrcIndexTransactionWithId {
            id: Nat::from(99_u64),
            transaction: IcrcIndexTransaction {
                burn: None,
                kind: "approve".to_string(),
                mint: None,
                approve: Some(IcrcIndexApprove {
                    fee: Some(Nat::from(10_u64)),
                    from: owner,
                    memo: Some(vec![0x01, 0x02]),
                    created_at_time: Some(1_700_000_000_000_000_000),
                    amount: Nat::from(250_u64),
                    expected_allowance: Some(Nat::from(200_u64)),
                    expires_at: Some(1_800_000_000_000_000_000),
                    spender,
                }),
                fee_collector: Some(IcrcIndexFeeCollector {
                    caller: Some(Principal::anonymous()),
                    fee_collector: Some(spender),
                    ts: Some(1_700_000_000_000_000_001),
                    mthd: Some("icrc2_approve".to_string()),
                }),
                authorized_mint: None,
                authorized_burn: None,
                timestamp: 1_700_000_000_000_000_002,
                transfer: None,
            },
        }],
        oldest_tx_id: Some(Nat::from(7_u64)),
    });

    let bytes =
        ic_platform::candid::contract::encode_one(&response).expect("encode index response");
    let decoded: IcrcIndexTransactionsResult =
        crate::candid_decode::decode_reply(&bytes).expect("decode index response");
    assert_eq!(decoded, response);

    let IcrcIndexTransactionsResult::Ok(decoded) = decoded else {
        panic!("expected successful index response");
    };
    let row = account_transaction_row(
        decoded
            .transactions
            .into_iter()
            .next()
            .expect("transaction row"),
    );

    assert_eq!(row.id, "99");
    assert_eq!(row.amount_base_units.as_deref(), Some("250"));
    assert_eq!(row.fee_base_units.as_deref(), Some("10"));
    assert_eq!(row.memo_hex.as_deref(), Some("0102"));
    assert_eq!(row.expected_allowance_base_units.as_deref(), Some("200"));
    assert_eq!(
        row.raw_transaction["approve"]["from"]["subaccount_hex"],
        json!("aa".repeat(32))
    );
    assert_eq!(
        row.raw_transaction["fee_collector"]["method"],
        json!("icrc2_approve")
    );
    assert_eq!(row.raw_transaction["transfer"], JsonValue::Null);
}

fn every_operation_transactions() -> IcrcIndexTransactions {
    let amount = Nat::parse(b"3402823669209384634633746074317682114561234").unwrap();
    let from = LedgerAccount {
        owner: Principal::anonymous(),
        subaccount: Some([0xff; 32]),
    };
    let to = LedgerAccount {
        owner: Principal::management_canister(),
        subaccount: None,
    };
    let transaction = IcrcIndexTransaction {
        kind: "transfer".to_string(),
        timestamp: u64::MAX,
        approve: Some(IcrcIndexApprove {
            fee: None,
            from,
            memo: Some(vec![]),
            created_at_time: Some(u64::MAX),
            amount: amount.clone(),
            expected_allowance: Some(amount.clone()),
            expires_at: None,
            spender: to,
        }),
        burn: Some(IcrcIndexBurn {
            from,
            memo: None,
            created_at_time: None,
            amount: amount.clone(),
            spender: None,
            fee: Some(Nat::from(0_u64)),
        }),
        mint: Some(IcrcIndexMint {
            to,
            memo: Some(vec![255]),
            created_at_time: None,
            amount: amount.clone(),
            fee: None,
        }),
        transfer: Some(IcrcIndexTransfer {
            to,
            fee: Some(amount.clone()),
            from,
            memo: Some(vec![0, 255]),
            created_at_time: None,
            amount: amount.clone(),
            spender: Some(to),
        }),
        fee_collector: Some(IcrcIndexFeeCollector {
            caller: Some(Principal::anonymous()),
            fee_collector: Some(to),
            ts: Some(u64::MAX),
            mthd: None,
        }),
        authorized_mint: Some(IcrcIndexAuthorizedMint {
            to,
            amount: amount.clone(),
            created_at_time: None,
            caller: None,
            mthd: Some("mint".to_string()),
            reason: Some("retained evidence".to_string()),
        }),
        authorized_burn: Some(IcrcIndexAuthorizedBurn {
            from,
            amount: amount.clone(),
            created_at_time: Some(u64::MAX),
            caller: Some(Principal::management_canister()),
            mthd: None,
            reason: None,
        }),
    };
    IcrcIndexTransactions {
        balance: amount.clone(),
        transactions: vec![IcrcIndexTransactionWithId {
            id: amount,
            transaction,
        }],
        oldest_tx_id: None,
    }
}
