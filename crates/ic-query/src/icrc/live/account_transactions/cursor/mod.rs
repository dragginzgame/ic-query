//! Module: icrc::live::account_transactions::cursor
//!
//! Responsibility: validate account-history pages and arbitrary-size transaction cursors.
//! Does not own: collection state, index wire decoding, transport, or reports.
//! Boundary: keeps public cursor normalization and snapshot ordering on canonical decimal text.

use crate::decimal::{
    compare_canonical_unsigned_decimal, is_canonical_unsigned_decimal, is_unsigned_decimal,
};
use crate::icrc::model::{IcrcAccountTransactionError, IcrcAccountTransactionRow};
use ic_platform::candid::contract::Nat;
use std::str::FromStr;

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
        if !is_canonical_unsigned_decimal(&transaction.id) {
            validate_transaction_cursor_text(&transaction.id).map_err(|error| error.to_string())?;
            return Err(format!(
                "transaction id {} is not canonical decimal text",
                transaction.id
            ));
        }
        if let Some(previous) = previous
            && compare_canonical_unsigned_decimal(&transaction.id, previous).is_ge()
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
        if let Some(value) = value
            && !is_canonical_unsigned_decimal(value)
        {
            validate_transaction_cursor_text(value)?;
            return Err(IcrcAccountTransactionError::InvalidPage {
                reason: format!("{field} {value:?} is not canonical unsigned decimal text"),
            });
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
        && requested_start
            .is_some_and(|start| compare_canonical_unsigned_decimal(&first.id, start).is_ge())
    {
        return Err(IcrcAccountTransactionError::InvalidPage {
            reason: "transaction is not below the exclusive requested cursor".to_string(),
        });
    }
    if let Some(last) = transactions.last()
        && oldest_transaction_id
            .is_some_and(|oldest| compare_canonical_unsigned_decimal(&last.id, oldest).is_lt())
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
    if !is_unsigned_decimal(value) {
        return Err(IcrcAccountTransactionError::InvalidCursor {
            value: value.to_string(),
            reason: "expected unsigned decimal text".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
