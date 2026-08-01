//! Module: icrc::live::account_transactions::wire::generic
//!
//! Responsibility: decode and losslessly project the generic ICRC index response.
//! Does not own: transport, index discovery, pagination state, ICP-index compatibility, or reports.
//! Boundary: keeps the current ICRC index Candid contract separate from deployed ICP wire types.

use super::super::{collection::AccountTransactionsPage, cursor::nat_text};
use super::{TransactionSummary, object, optional_blob_json, optional_json};
use crate::{
    hex::hex_bytes,
    icrc::{
        ledger::IcrcAccount,
        model::{IcrcAccountRow, IcrcAccountTransactionError, IcrcAccountTransactionRow},
    },
};
use candid::{CandidType, Deserialize, Nat, Principal};
use serde_json::Value as JsonValue;

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum IcrcIndexTransactionsResult {
    Ok(IcrcIndexTransactions),
    Err(IcrcIndexTransactionsError),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransactions {
    balance: Nat,
    transactions: Vec<IcrcIndexTransactionWithId>,
    oldest_tx_id: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransactionsError {
    message: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransactionWithId {
    id: Nat,
    transaction: IcrcIndexTransaction,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
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
struct IcrcIndexApprove {
    fee: Option<Nat>,
    from: IcrcAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    expected_allowance: Option<Nat>,
    expires_at: Option<u64>,
    spender: IcrcAccount,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexBurn {
    from: IcrcAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    spender: Option<IcrcAccount>,
    fee: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexMint {
    to: IcrcAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    fee: Option<Nat>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexTransfer {
    to: IcrcAccount,
    fee: Option<Nat>,
    from: IcrcAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    spender: Option<IcrcAccount>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexFeeCollector {
    caller: Option<Principal>,
    fee_collector: Option<IcrcAccount>,
    ts: Option<u64>,
    mthd: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexAuthorizedMint {
    to: IcrcAccount,
    amount: Nat,
    created_at_time: Option<u64>,
    caller: Option<Principal>,
    mthd: Option<String>,
    reason: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcIndexAuthorizedBurn {
    from: IcrcAccount,
    amount: Nat,
    created_at_time: Option<u64>,
    caller: Option<Principal>,
    mthd: Option<String>,
    reason: Option<String>,
}

pub(super) fn decode(
    bytes: &[u8],
    index_canister: &Principal,
) -> Result<Result<AccountTransactionsPage, IcrcAccountTransactionError>, candid::Error> {
    candid::decode_one::<IcrcIndexTransactionsResult>(bytes)
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
        raw_transaction: transaction_json(&transaction.transaction),
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

fn account_row(account: &IcrcAccount) -> IcrcAccountRow {
    IcrcAccountRow {
        owner: Some(account.owner.to_text()),
        subaccount_hex: account.subaccount.as_deref().map(hex_bytes),
        account_identifier: None,
    }
}

fn transaction_json(transaction: &IcrcIndexTransaction) -> JsonValue {
    object([
        ("kind", JsonValue::String(transaction.kind.clone())),
        (
            "timestamp_unix_nanos",
            JsonValue::String(transaction.timestamp.to_string()),
        ),
        (
            "burn",
            optional_json(transaction.burn.as_ref().map(burn_json)),
        ),
        (
            "mint",
            optional_json(transaction.mint.as_ref().map(mint_json)),
        ),
        (
            "approve",
            optional_json(transaction.approve.as_ref().map(approve_json)),
        ),
        (
            "fee_collector",
            optional_json(transaction.fee_collector.as_ref().map(fee_collector_json)),
        ),
        (
            "authorized_mint",
            optional_json(
                transaction
                    .authorized_mint
                    .as_ref()
                    .map(authorized_mint_json),
            ),
        ),
        (
            "authorized_burn",
            optional_json(
                transaction
                    .authorized_burn
                    .as_ref()
                    .map(authorized_burn_json),
            ),
        ),
        (
            "transfer",
            optional_json(transaction.transfer.as_ref().map(transfer_json)),
        ),
    ])
}

fn approve_json(approve: &IcrcIndexApprove) -> JsonValue {
    object([
        ("fee_base_units", optional_nat_json(approve.fee.as_ref())),
        ("from", account_json(&approve.from)),
        ("memo_hex", optional_blob_json(approve.memo.as_deref())),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(approve.created_at_time),
        ),
        ("amount_base_units", nat_json(&approve.amount)),
        (
            "expected_allowance_base_units",
            optional_nat_json(approve.expected_allowance.as_ref()),
        ),
        (
            "expires_at_unix_nanos",
            optional_u64_json(approve.expires_at),
        ),
        ("spender", account_json(&approve.spender)),
    ])
}

fn burn_json(burn: &IcrcIndexBurn) -> JsonValue {
    object([
        ("from", account_json(&burn.from)),
        ("memo_hex", optional_blob_json(burn.memo.as_deref())),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(burn.created_at_time),
        ),
        ("amount_base_units", nat_json(&burn.amount)),
        (
            "spender",
            optional_json(burn.spender.as_ref().map(account_json)),
        ),
        ("fee_base_units", optional_nat_json(burn.fee.as_ref())),
    ])
}

fn mint_json(mint: &IcrcIndexMint) -> JsonValue {
    object([
        ("to", account_json(&mint.to)),
        ("memo_hex", optional_blob_json(mint.memo.as_deref())),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(mint.created_at_time),
        ),
        ("amount_base_units", nat_json(&mint.amount)),
        ("fee_base_units", optional_nat_json(mint.fee.as_ref())),
    ])
}

fn transfer_json(transfer: &IcrcIndexTransfer) -> JsonValue {
    object([
        ("to", account_json(&transfer.to)),
        ("fee_base_units", optional_nat_json(transfer.fee.as_ref())),
        ("from", account_json(&transfer.from)),
        ("memo_hex", optional_blob_json(transfer.memo.as_deref())),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(transfer.created_at_time),
        ),
        ("amount_base_units", nat_json(&transfer.amount)),
        (
            "spender",
            optional_json(transfer.spender.as_ref().map(account_json)),
        ),
    ])
}

fn fee_collector_json(fee_collector: &IcrcIndexFeeCollector) -> JsonValue {
    object([
        (
            "caller",
            optional_json(
                fee_collector
                    .caller
                    .as_ref()
                    .map(|caller| JsonValue::String(caller.to_text())),
            ),
        ),
        (
            "fee_collector",
            optional_json(fee_collector.fee_collector.as_ref().map(account_json)),
        ),
        ("timestamp_unix_nanos", optional_u64_json(fee_collector.ts)),
        (
            "method",
            optional_json(fee_collector.mthd.clone().map(JsonValue::String)),
        ),
    ])
}

fn authorized_mint_json(mint: &IcrcIndexAuthorizedMint) -> JsonValue {
    object([
        ("to", account_json(&mint.to)),
        ("amount_base_units", nat_json(&mint.amount)),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(mint.created_at_time),
        ),
        (
            "caller",
            optional_json(
                mint.caller
                    .as_ref()
                    .map(|caller| JsonValue::String(caller.to_text())),
            ),
        ),
        (
            "method",
            optional_json(mint.mthd.clone().map(JsonValue::String)),
        ),
        (
            "reason",
            optional_json(mint.reason.clone().map(JsonValue::String)),
        ),
    ])
}

fn authorized_burn_json(burn: &IcrcIndexAuthorizedBurn) -> JsonValue {
    object([
        ("from", account_json(&burn.from)),
        ("amount_base_units", nat_json(&burn.amount)),
        (
            "created_at_time_unix_nanos",
            optional_u64_json(burn.created_at_time),
        ),
        (
            "caller",
            optional_json(
                burn.caller
                    .as_ref()
                    .map(|caller| JsonValue::String(caller.to_text())),
            ),
        ),
        (
            "method",
            optional_json(burn.mthd.clone().map(JsonValue::String)),
        ),
        (
            "reason",
            optional_json(burn.reason.clone().map(JsonValue::String)),
        ),
    ])
}

fn account_json(account: &IcrcAccount) -> JsonValue {
    object([
        ("owner", JsonValue::String(account.owner.to_text())),
        (
            "subaccount_hex",
            optional_json(
                account
                    .subaccount
                    .as_deref()
                    .map(hex_bytes)
                    .map(JsonValue::String),
            ),
        ),
    ])
}

fn nat_json(value: &Nat) -> JsonValue {
    JsonValue::String(nat_text(value))
}

fn optional_nat_json(value: Option<&Nat>) -> JsonValue {
    optional_json(value.map(nat_json))
}

fn optional_u64_json(value: Option<u64>) -> JsonValue {
    optional_json(value.map(|value| JsonValue::String(value.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn current_index_wire_shape_round_trips_and_projects_approve_losslessly() {
        let owner = IcrcAccount {
            owner: Principal::anonymous(),
            subaccount: Some(vec![0xaa; 32]),
        };
        let spender = IcrcAccount {
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
                        spender: spender.clone(),
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

        let bytes = candid::encode_one(&response).expect("encode index response");
        let decoded: IcrcIndexTransactionsResult =
            candid::decode_one(&bytes).expect("decode index response");
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
}
