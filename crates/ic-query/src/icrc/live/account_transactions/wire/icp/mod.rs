//! Module: icrc::live::account_transactions::wire::icp
//!
//! Responsibility: decode and losslessly project the deployed ICP index response.
//! Does not own: transport, index discovery, pagination state, generic ICRC-index decoding, or reports.
//! Boundary: preserves ICP account identifiers and legacy operation fields without conflating wire contracts.

use super::super::collection::AccountTransactionsPage;
use super::{
    TransactionSummary,
    json::{Decimal, Hex, json_record},
};
use crate::{
    hex::hex_bytes,
    icrc::model::{IcrcAccountRow, IcrcAccountTransactionError, IcrcAccountTransactionRow},
};
use ic_platform::candid::contract::{CandidType, Principal, serde::Deserialize};
use serde::{Serialize, Serializer};

type IcpIndexTransactionsResult = Result<IcpIndexTransactions, IcpIndexTransactionsError>;

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcpIndexTransactionsError {
    message: String,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcpIndexTransactions {
    balance: u64,
    transactions: Vec<IcpIndexTransactionWithId>,
    oldest_tx_id: Option<u64>,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcpIndexTransactionWithId {
    id: u64,
    transaction: IcpIndexTransaction,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcpIndexTransaction {
    memo: u64,
    icrc1_memo: Option<Vec<u8>>,
    operation: IcpIndexOperation,
    created_at_time: Option<IcpIndexTimestamp>,
    timestamp: Option<IcpIndexTimestamp>,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
enum IcpIndexOperation {
    Approve {
        fee: IcpIndexTokens,
        from: String,
        allowance: IcpIndexTokens,
        expires_at: Option<IcpIndexTimestamp>,
        spender: String,
        expected_allowance: Option<IcpIndexTokens>,
    },
    Burn {
        from: String,
        amount: IcpIndexTokens,
        spender: Option<String>,
    },
    Mint {
        to: String,
        amount: IcpIndexTokens,
    },
    Transfer {
        to: String,
        fee: IcpIndexTokens,
        from: String,
        amount: IcpIndexTokens,
        spender: Option<String>,
    },
}

#[derive(CandidType, Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcpIndexTimestamp {
    timestamp_nanos: u64,
}

#[derive(CandidType, Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
struct IcpIndexTokens {
    e8s: u64,
}

pub(super) fn decode(
    bytes: &[u8],
    index_canister: &Principal,
) -> Result<
    Result<AccountTransactionsPage, IcrcAccountTransactionError>,
    ic_platform::candid::contract::Error,
> {
    crate::candid_decode::decode_reply::<IcpIndexTransactionsResult>(bytes)
        .map(|result| account_transactions_page(result, index_canister))
}

fn account_transactions_page(
    result: IcpIndexTransactionsResult,
    index_canister: &Principal,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    let transactions = result.map_err(|error| IcrcAccountTransactionError::IndexQuery {
        index_canister_id: index_canister.to_text(),
        message: error.message,
    })?;
    Ok(AccountTransactionsPage {
        balance: transactions.balance.to_string(),
        oldest_transaction_id: transactions.oldest_tx_id.map(|id| id.to_string()),
        transactions: transactions
            .transactions
            .into_iter()
            .map(account_transaction_row)
            .collect(),
    })
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Consumes each decoded native transaction while projecting one owned report row."
)]
fn account_transaction_row(transaction: IcpIndexTransactionWithId) -> IcrcAccountTransactionRow {
    let (kind, mut summary) = operation_summary(&transaction.transaction.operation);
    summary.memo_hex = transaction.transaction.icrc1_memo.as_deref().map(hex_bytes);
    summary.created_at_time_unix_nanos = transaction
        .transaction
        .created_at_time
        .map(|timestamp| timestamp.timestamp_nanos.to_string());
    summary.into_row(
        transaction.id.to_string(),
        kind.to_string(),
        transaction
            .transaction
            .timestamp
            .map(|timestamp| timestamp.timestamp_nanos.to_string()),
        serde_json::to_value(&transaction.transaction)
            .expect("ICP index transaction JSON serialization is infallible"),
    )
}

fn operation_summary(operation: &IcpIndexOperation) -> (&'static str, TransactionSummary) {
    match operation {
        IcpIndexOperation::Approve {
            fee,
            from,
            allowance,
            expires_at,
            spender,
            expected_allowance,
        } => (
            "approve",
            TransactionSummary {
                amount_base_units: Some(allowance.e8s.to_string()),
                fee_base_units: Some(fee.e8s.to_string()),
                from: Some(account_identifier_row(from)),
                spender: Some(account_identifier_row(spender)),
                expires_at_unix_nanos: expires_at
                    .map(|timestamp| timestamp.timestamp_nanos.to_string()),
                expected_allowance_base_units: expected_allowance
                    .map(|tokens| tokens.e8s.to_string()),
                ..TransactionSummary::default()
            },
        ),
        IcpIndexOperation::Burn {
            from,
            amount,
            spender,
        } => (
            "burn",
            TransactionSummary {
                amount_base_units: Some(amount.e8s.to_string()),
                from: Some(account_identifier_row(from)),
                spender: spender.as_deref().map(account_identifier_row),
                ..TransactionSummary::default()
            },
        ),
        IcpIndexOperation::Mint { to, amount } => (
            "mint",
            TransactionSummary {
                amount_base_units: Some(amount.e8s.to_string()),
                to: Some(account_identifier_row(to)),
                ..TransactionSummary::default()
            },
        ),
        IcpIndexOperation::Transfer {
            to,
            fee,
            from,
            amount,
            spender,
        } => (
            "transfer",
            TransactionSummary {
                amount_base_units: Some(amount.e8s.to_string()),
                fee_base_units: Some(fee.e8s.to_string()),
                from: Some(account_identifier_row(from)),
                to: Some(account_identifier_row(to)),
                spender: spender.as_deref().map(account_identifier_row),
                ..TransactionSummary::default()
            },
        ),
    }
}

json_record!(IcpIndexTransaction, transaction, {
    "memo" => Decimal(transaction.memo),
    "icrc1_memo_hex" => transaction.icrc1_memo.as_deref().map(Hex),
    "operation" => transaction.operation,
    "created_at_time_unix_nanos" => transaction.created_at_time,
    "timestamp_unix_nanos" => transaction.timestamp,
});

impl Serialize for IcpIndexTimestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Decimal(self.timestamp_nanos).serialize(serializer)
    }
}

impl Serialize for IcpIndexTokens {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Decimal(self.e8s).serialize(serializer)
    }
}

impl Serialize for IcpIndexOperation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Approve {
                fee,
                from,
                allowance,
                expires_at,
                spender,
                expected_allowance,
            } => json_record!(serializer, {
                "kind" => "approve",
                "fee_base_units" => fee,
                "from_account_identifier" => from,
                "allowance_base_units" => allowance,
                "expires_at_unix_nanos" => expires_at,
                "spender_account_identifier" => spender,
                "expected_allowance_base_units" => expected_allowance,
            }),
            Self::Burn {
                from,
                amount,
                spender,
            } => json_record!(serializer, {
                "kind" => "burn",
                "from_account_identifier" => from,
                "amount_base_units" => amount,
                "spender_account_identifier" => spender,
            }),
            Self::Mint { to, amount } => json_record!(serializer, {
                "kind" => "mint",
                "to_account_identifier" => to,
                "amount_base_units" => amount,
            }),
            Self::Transfer {
                to,
                fee,
                from,
                amount,
                spender,
            } => json_record!(serializer, {
                "kind" => "transfer",
                "to_account_identifier" => to,
                "fee_base_units" => fee,
                "from_account_identifier" => from,
                "amount_base_units" => amount,
                "spender_account_identifier" => spender,
            }),
        }
    }
}

fn account_identifier_row(account_identifier: &str) -> IcrcAccountRow {
    IcrcAccountRow::Legacy {
        account_identifier: account_identifier.to_string(),
    }
}

#[cfg(test)]
mod tests;
