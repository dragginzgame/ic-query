//! Module: icrc::live::account_transactions::cursor
//!
//! Responsibility: validate account-history pages and arbitrary-size transaction cursors.
//! Does not own: collection state, index wire decoding, transport, or reports.
//! Boundary: keeps public cursor normalization and snapshot ordering on canonical decimal text.

use crate::icrc::model::{IcrcAccountTransactionError, IcrcAccountTransactionRow};
use candid::Nat;
use std::{cmp::Ordering, str::FromStr};

pub(in crate::icrc) fn normalize_transaction_cursor(
    value: &str,
) -> Result<String, IcrcAccountTransactionError> {
    validate_transaction_cursor_text(value)?;
    let normalized = value.trim_start_matches('0');
    Ok(if normalized.is_empty() {
        "0"
    } else {
        normalized
    }
    .to_string())
}

pub(in crate::icrc) fn validate_canonical_account_transactions(
    transactions: &[IcrcAccountTransactionRow],
) -> Result<(), String> {
    let mut previous = None;
    for transaction in transactions {
        validate_transaction_cursor_text(&transaction.id).map_err(|error| error.to_string())?;
        if transaction.id.len() > 1 && transaction.id.starts_with('0') {
            return Err(format!(
                "transaction id {} is not canonical decimal text",
                transaction.id
            ));
        }
        if let Some(previous) = previous
            && compare_canonical_decimal(&transaction.id, previous).is_ge()
        {
            return Err("transactions are not unique newest-first rows".to_string());
        }
        previous = Some(transaction.id.as_str());
    }
    Ok(())
}

/// Validate page metadata and rows before reporting or ingesting an index page.
/// The requested start is already canonical from request normalization or an accepted page.
pub(in crate::icrc) fn validate_account_transaction_page(
    transactions: &[IcrcAccountTransactionRow],
    requested_start: Option<&str>,
    oldest_transaction_id: Option<&str>,
    next_start: Option<&str>,
) -> Result<(), IcrcAccountTransactionError> {
    for (value, field) in [
        (next_start, "next_start"),
        (oldest_transaction_id, "oldest_transaction_id"),
    ] {
        if let Some(value) = value {
            validate_transaction_cursor_text(value)?;
            if value.len() > 1 && value.starts_with('0') {
                return Err(IcrcAccountTransactionError::InvalidPage {
                    reason: format!("{field} {value:?} is not canonical unsigned decimal text"),
                });
            }
        }
    }
    validate_canonical_account_transactions(transactions)
        .map_err(|reason| IcrcAccountTransactionError::InvalidPage { reason })?;
    if next_start
        != transactions
            .last()
            .map(|transaction| transaction.id.as_str())
    {
        return Err(IcrcAccountTransactionError::InvalidPage {
            reason: "next cursor does not match the oldest returned transaction".to_string(),
        });
    }
    if let Some(first) = transactions.first()
        && requested_start.is_some_and(|start| compare_canonical_decimal(&first.id, start).is_ge())
    {
        return Err(IcrcAccountTransactionError::InvalidPage {
            reason: "transaction is not below the exclusive requested cursor".to_string(),
        });
    }
    if let Some(last) = transactions.last()
        && oldest_transaction_id
            .is_some_and(|oldest| compare_canonical_decimal(&last.id, oldest).is_lt())
    {
        return Err(IcrcAccountTransactionError::InvalidPage {
            reason: "transaction is below the index's oldest transaction id".to_string(),
        });
    }
    Ok(())
}

pub(super) fn parse_transaction_cursor(value: &str) -> Result<Nat, IcrcAccountTransactionError> {
    validate_transaction_cursor_text(value)?;
    Nat::from_str(value).map_err(|error| IcrcAccountTransactionError::InvalidCursor {
        value: value.to_string(),
        reason: error.to_string(),
    })
}

pub(in crate::icrc) fn validate_transaction_cursor_text(
    value: &str,
) -> Result<(), IcrcAccountTransactionError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(IcrcAccountTransactionError::InvalidCursor {
            value: value.to_string(),
            reason: "expected unsigned decimal text".to_string(),
        });
    }
    Ok(())
}

pub(super) fn compare_canonical_decimal(left: &str, right: &str) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

#[cfg(test)]
mod tests {
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
}
