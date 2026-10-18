//! Module: icrc::live::account_transactions::wire::generic
//!
//! Responsibility: decode and losslessly project the generic ICRC index response.
//! Does not own: transport, index discovery, pagination state, ICP-index compatibility, or reports.
//! Boundary: keeps the current ICRC index Candid contract separate from deployed ICP wire types.

use super::super::collection::AccountTransactionsPage;
use super::{
    TransactionSummary,
    json::{Account, Decimal, Hex, json_record},
};
use crate::{
    hex::hex_bytes,
    icrc::{
        ledger::nat_text,
        model::{IcrcAccountRow, IcrcAccountTransactionError, IcrcAccountTransactionRow},
    },
};
use ic_platform::candid::contract::{CandidType, Nat, Principal, serde::Deserialize};
use ic_platform::icrc_ledger_types::Account as LedgerAccount;

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
enum IcrcIndexTransactionsResult {
    Ok(IcrcIndexTransactions),
    Err(IcrcIndexTransactionsError),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexTransactions {
    balance: Nat,
    transactions: Vec<IcrcIndexTransactionWithId>,
    oldest_tx_id: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexTransactionsError {
    message: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexTransactionWithId {
    id: Nat,
    transaction: IcrcIndexTransaction,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexTransaction {
    burn: Option<IcrcIndexBurn>,
    kind: String,
    mint: Option<IcrcIndexMint>,
    approve: Option<IcrcIndexApprove>,
    fee_collector: Option<IcrcIndexFeeCollector>,
    authorized_mint: Option<IcrcIndexAuthorizedMint>,
    authorized_burn: Option<IcrcIndexAuthorizedBurn>,
    timestamp: u64,
    transfer: Option<IcrcIndexTransfer>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexApprove {
    fee: Option<Nat>,
    from: LedgerAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    expected_allowance: Option<Nat>,
    expires_at: Option<u64>,
    spender: LedgerAccount,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexBurn {
    from: LedgerAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    spender: Option<LedgerAccount>,
    fee: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexMint {
    to: LedgerAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    fee: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexTransfer {
    to: LedgerAccount,
    fee: Option<Nat>,
    from: LedgerAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    spender: Option<LedgerAccount>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexFeeCollector {
    caller: Option<Principal>,
    fee_collector: Option<LedgerAccount>,
    ts: Option<u64>,
    mthd: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexAuthorizedMint {
    to: LedgerAccount,
    amount: Nat,
    created_at_time: Option<u64>,
    caller: Option<Principal>,
    mthd: Option<String>,
    reason: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexAuthorizedBurn {
    from: LedgerAccount,
    amount: Nat,
    created_at_time: Option<u64>,
    caller: Option<Principal>,
    mthd: Option<String>,
    reason: Option<String>,
}

pub(super) fn decode(
    bytes: &[u8],
    index_canister: &Principal,
) -> Result<
    Result<AccountTransactionsPage, IcrcAccountTransactionError>,
    ic_platform::candid::contract::Error,
> {
    crate::candid_decode::decode_reply::<IcrcIndexTransactionsResult>(bytes)
        .map(|result| account_transactions_page(result, index_canister))
}

fn account_transactions_page(
    result: IcrcIndexTransactionsResult,
    index_canister: &Principal,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    let transactions = match result {
        IcrcIndexTransactionsResult::Ok(transactions) => transactions,
        IcrcIndexTransactionsResult::Err(error) => {
            return Err(IcrcAccountTransactionError::IndexQuery {
                index_canister_id: index_canister.to_text(),
                message: error.message,
            });
        }
    };
    Ok(AccountTransactionsPage {
        balance: nat_text(&transactions.balance),
        oldest_transaction_id: transactions.oldest_tx_id.as_ref().map(nat_text),
        next_start: transactions
            .transactions
            .last()
            .map(|transaction| nat_text(&transaction.id)),
        transactions: transactions
            .transactions
            .into_iter()
            .map(account_transaction_row)
            .collect(),
    })
}

fn account_transaction_row(transaction: IcrcIndexTransactionWithId) -> IcrcAccountTransactionRow {
    let summary = transaction_summary(&transaction.transaction);
    IcrcAccountTransactionRow {
        id: nat_text(&transaction.id),
        kind: transaction.transaction.kind.clone(),
        timestamp_unix_nanos: Some(transaction.transaction.timestamp.to_string()),
        amount_base_units: summary.amount_base_units,
        fee_base_units: summary.fee_base_units,
        from: summary.from,
        to: summary.to,
        spender: summary.spender,
        memo_hex: summary.memo_hex,
        created_at_time_unix_nanos: summary.created_at_time_unix_nanos,
        expires_at_unix_nanos: summary.expires_at_unix_nanos,
        expected_allowance_base_units: summary.expected_allowance_base_units,
        raw_transaction: serde_json::to_value(&transaction.transaction)
            .expect("index transaction JSON serialization is infallible"),
    }
}

fn transaction_summary(transaction: &IcrcIndexTransaction) -> TransactionSummary {
    if let Some(transfer) = transaction.transfer.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&transfer.amount)),
            fee_base_units: transfer.fee.as_ref().map(nat_text),
            from: Some(account_row(&transfer.from)),
            to: Some(account_row(&transfer.to)),
            spender: transfer.spender.as_ref().map(account_row),
            memo_hex: transfer.memo.as_deref().map(hex_bytes),
            created_at_time_unix_nanos: transfer.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    if let Some(mint) = transaction.mint.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&mint.amount)),
            fee_base_units: mint.fee.as_ref().map(nat_text),
            to: Some(account_row(&mint.to)),
            memo_hex: mint.memo.as_deref().map(hex_bytes),
            created_at_time_unix_nanos: mint.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    if let Some(burn) = transaction.burn.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&burn.amount)),
            fee_base_units: burn.fee.as_ref().map(nat_text),
            from: Some(account_row(&burn.from)),
            spender: burn.spender.as_ref().map(account_row),
            memo_hex: burn.memo.as_deref().map(hex_bytes),
            created_at_time_unix_nanos: burn.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    if let Some(approve) = transaction.approve.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&approve.amount)),
            fee_base_units: approve.fee.as_ref().map(nat_text),
            from: Some(account_row(&approve.from)),
            spender: Some(account_row(&approve.spender)),
            memo_hex: approve.memo.as_deref().map(hex_bytes),
            created_at_time_unix_nanos: approve.created_at_time.map(|time| time.to_string()),
            expires_at_unix_nanos: approve.expires_at.map(|time| time.to_string()),
            expected_allowance_base_units: approve.expected_allowance.as_ref().map(nat_text),
            ..TransactionSummary::default()
        };
    }
    if let Some(mint) = transaction.authorized_mint.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&mint.amount)),
            to: Some(account_row(&mint.to)),
            created_at_time_unix_nanos: mint.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    if let Some(burn) = transaction.authorized_burn.as_ref() {
        return TransactionSummary {
            amount_base_units: Some(nat_text(&burn.amount)),
            from: Some(account_row(&burn.from)),
            created_at_time_unix_nanos: burn.created_at_time.map(|time| time.to_string()),
            ..TransactionSummary::default()
        };
    }
    TransactionSummary::default()
}

fn account_row(account: &LedgerAccount) -> IcrcAccountRow {
    IcrcAccountRow {
        owner: Some(account.owner.to_text()),
        subaccount_hex: account
            .subaccount
            .as_ref()
            .map(<[u8; 32]>::as_slice)
            .map(hex_bytes),
        account_identifier: None,
    }
}

json_record!(IcrcIndexTransaction, transaction, {
    "kind" => transaction.kind,
    "timestamp_unix_nanos" => Decimal(transaction.timestamp),
    "burn" => transaction.burn,
    "mint" => transaction.mint,
    "approve" => transaction.approve,
    "fee_collector" => transaction.fee_collector,
    "authorized_mint" => transaction.authorized_mint,
    "authorized_burn" => transaction.authorized_burn,
    "transfer" => transaction.transfer,
});

json_record!(IcrcIndexApprove, approve, {
    "fee_base_units" => approve.fee.as_ref().map(|value| Decimal(&value.0)),
    "from" => Account(&approve.from),
    "memo_hex" => approve.memo.as_deref().map(Hex),
    "created_at_time_unix_nanos" => approve.created_at_time.map(Decimal),
    "amount_base_units" => Decimal(&approve.amount.0),
    "expected_allowance_base_units" => approve.expected_allowance.as_ref().map(|value| Decimal(&value.0)),
    "expires_at_unix_nanos" => approve.expires_at.map(Decimal),
    "spender" => Account(&approve.spender),
});

json_record!(IcrcIndexBurn, burn, {
    "from" => Account(&burn.from),
    "memo_hex" => burn.memo.as_deref().map(Hex),
    "created_at_time_unix_nanos" => burn.created_at_time.map(Decimal),
    "amount_base_units" => Decimal(&burn.amount.0),
    "spender" => burn.spender.as_ref().map(Account),
    "fee_base_units" => burn.fee.as_ref().map(|value| Decimal(&value.0)),
});

json_record!(IcrcIndexMint, mint, {
    "to" => Account(&mint.to),
    "memo_hex" => mint.memo.as_deref().map(Hex),
    "created_at_time_unix_nanos" => mint.created_at_time.map(Decimal),
    "amount_base_units" => Decimal(&mint.amount.0),
    "fee_base_units" => mint.fee.as_ref().map(|value| Decimal(&value.0)),
});

json_record!(IcrcIndexTransfer, transfer, {
    "to" => Account(&transfer.to),
    "fee_base_units" => transfer.fee.as_ref().map(|value| Decimal(&value.0)),
    "from" => Account(&transfer.from),
    "memo_hex" => transfer.memo.as_deref().map(Hex),
    "created_at_time_unix_nanos" => transfer.created_at_time.map(Decimal),
    "amount_base_units" => Decimal(&transfer.amount.0),
    "spender" => transfer.spender.as_ref().map(Account),
});

json_record!(IcrcIndexFeeCollector, collector, {
    "caller" => collector.caller.as_ref().map(Principal::to_text),
    "fee_collector" => collector.fee_collector.as_ref().map(Account),
    "timestamp_unix_nanos" => collector.ts.map(Decimal),
    "method" => collector.mthd,
});

json_record!(IcrcIndexAuthorizedMint, mint, {
    "to" => Account(&mint.to),
    "amount_base_units" => Decimal(&mint.amount.0),
    "created_at_time_unix_nanos" => mint.created_at_time.map(Decimal),
    "caller" => mint.caller.as_ref().map(Principal::to_text),
    "method" => mint.mthd,
    "reason" => mint.reason,
});

json_record!(IcrcIndexAuthorizedBurn, burn, {
    "from" => Account(&burn.from),
    "amount_base_units" => Decimal(&burn.amount.0),
    "created_at_time_unix_nanos" => burn.created_at_time.map(Decimal),
    "caller" => burn.caller.as_ref().map(Principal::to_text),
    "method" => burn.mthd,
    "reason" => burn.reason,
});

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value as JsonValue, json};

    #[test]
    fn every_operation_preserves_exact_decimal_blob_and_nullable_json_fields() {
        let response = every_operation_response();
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

    fn every_operation_response() -> IcrcIndexTransactionsResult {
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
        IcrcIndexTransactionsResult::Ok(IcrcIndexTransactions {
            balance: amount.clone(),
            transactions: vec![IcrcIndexTransactionWithId {
                id: amount,
                transaction,
            }],
            oldest_tx_id: None,
        })
    }
}
