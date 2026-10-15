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
use candid::{CandidType, Deserialize, Principal};
use serde::{Serialize, Serializer};

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum IcpIndexTransactionsResult {
    Ok(IcpIndexTransactions),
    Err(IcpIndexTransactionsError),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTransactionsError {
    message: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTransactions {
    balance: u64,
    transactions: Vec<IcpIndexTransactionWithId>,
    oldest_tx_id: Option<u64>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTransactionWithId {
    id: u64,
    transaction: IcpIndexTransaction,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTransaction {
    memo: u64,
    icrc1_memo: Option<Vec<u8>>,
    operation: IcpIndexOperation,
    created_at_time: Option<IcpIndexTimestamp>,
    timestamp: Option<IcpIndexTimestamp>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
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
struct IcpIndexTimestamp {
    timestamp_nanos: u64,
}

#[derive(CandidType, Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
struct IcpIndexTokens {
    e8s: u64,
}

pub(super) fn decode(
    bytes: &[u8],
    index_canister: &Principal,
) -> Result<Result<AccountTransactionsPage, IcrcAccountTransactionError>, candid::Error> {
    crate::candid_decode::decode_reply::<IcpIndexTransactionsResult>(bytes)
        .map(|result| account_transactions_page(result, index_canister))
}

fn account_transactions_page(
    result: IcpIndexTransactionsResult,
    index_canister: &Principal,
) -> Result<AccountTransactionsPage, IcrcAccountTransactionError> {
    let transactions = match result {
        IcpIndexTransactionsResult::Ok(transactions) => transactions,
        IcpIndexTransactionsResult::Err(error) => {
            return Err(IcrcAccountTransactionError::IndexQuery {
                index_canister_id: index_canister.to_text(),
                message: error.message,
            });
        }
    };
    Ok(AccountTransactionsPage {
        balance: transactions.balance.to_string(),
        oldest_transaction_id: transactions.oldest_tx_id.map(|id| id.to_string()),
        next_start: transactions
            .transactions
            .last()
            .map(|transaction| transaction.id.to_string()),
        transactions: transactions
            .transactions
            .into_iter()
            .map(account_transaction_row)
            .collect(),
    })
}

fn account_transaction_row(transaction: IcpIndexTransactionWithId) -> IcrcAccountTransactionRow {
    let (kind, mut summary) = operation_summary(&transaction.transaction.operation);
    summary.memo_hex = transaction.transaction.icrc1_memo.as_deref().map(hex_bytes);
    summary.created_at_time_unix_nanos = transaction
        .transaction
        .created_at_time
        .map(|timestamp| timestamp.timestamp_nanos.to_string());
    IcrcAccountTransactionRow {
        id: transaction.id.to_string(),
        kind: kind.to_string(),
        timestamp_unix_nanos: transaction
            .transaction
            .timestamp
            .map(|timestamp| timestamp.timestamp_nanos.to_string()),
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
            .expect("ICP index transaction JSON serialization is infallible"),
    }
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
    IcrcAccountRow {
        owner: None,
        subaccount_hex: None,
        account_identifier: Some(account_identifier.to_string()),
    }
}

#[cfg(test)]
mod tests {
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
            let bytes = candid::encode_one(&response).unwrap();
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
        let bytes = candid::encode_one(&response).expect("encode ICP index response");

        let page = decode(&bytes, &Principal::management_canister())
            .expect("decode ICP wire shape")
            .expect("successful ICP index response");
        let row = &page.transactions[0];

        assert_eq!(page.balance, "500");
        assert_eq!(page.oldest_transaction_id.as_deref(), Some("7"));
        assert_eq!(page.next_start.as_deref(), Some("99"));
        assert_eq!(row.kind, "transfer");
        assert_eq!(row.amount_base_units.as_deref(), Some("250"));
        assert_eq!(row.fee_base_units.as_deref(), Some("10"));
        assert_eq!(
            row.from
                .as_ref()
                .and_then(|account| account.account_identifier.as_deref()),
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
}
