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

type IcrcIndexTransactionsResult = Result<IcrcIndexTransactions, IcrcIndexTransactionsError>;

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexTransactions {
    balance: Nat,
    transactions: Vec<IcrcIndexTransactionWithId>,
    oldest_tx_id: Option<Nat>,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexTransactionsError {
    message: String,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexTransactionWithId {
    id: Nat,
    transaction: IcrcIndexTransaction,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
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

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
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

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexBurn {
    from: LedgerAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    spender: Option<LedgerAccount>,
    fee: Option<Nat>,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexMint {
    to: LedgerAccount,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    fee: Option<Nat>,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
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

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexFeeCollector {
    caller: Option<Principal>,
    fee_collector: Option<LedgerAccount>,
    ts: Option<u64>,
    mthd: Option<String>,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcrcIndexAuthorizedMint {
    to: LedgerAccount,
    amount: Nat,
    created_at_time: Option<u64>,
    caller: Option<Principal>,
    mthd: Option<String>,
    reason: Option<String>,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
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
    let transactions = result.map_err(|error| IcrcAccountTransactionError::IndexQuery {
        index_canister_id: index_canister.to_text(),
        message: error.message,
    })?;
    Ok(AccountTransactionsPage {
        balance: nat_text(&transactions.balance),
        oldest_transaction_id: transactions.oldest_tx_id.as_ref().map(nat_text),
        transactions: transactions
            .transactions
            .into_iter()
            .map(account_transaction_row)
            .collect(),
    })
}

fn account_transaction_row(transaction: IcrcIndexTransactionWithId) -> IcrcAccountTransactionRow {
    let summary = transaction_summary(&transaction.transaction);
    summary.into_row(
        nat_text(&transaction.id),
        transaction.transaction.kind.clone(),
        Some(transaction.transaction.timestamp.to_string()),
        serde_json::to_value(&transaction.transaction)
            .expect("index transaction JSON serialization is infallible"),
    )
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
    IcrcAccountRow::Structured {
        owner: account.owner.to_text(),
        subaccount_hex: account
            .subaccount
            .as_ref()
            .map(<[u8; 32]>::as_slice)
            .map(hex_bytes),
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
mod tests;
