//! Module: icrc::live::account_transactions::cursor
//!
//! Responsibility: validate and compare arbitrary-size account-transaction cursors.
//! Does not own: collection state, index wire decoding, transport, or reports.
//! Boundary: keeps public cursor normalization and snapshot ordering on canonical decimal text.

use crate::icrc::model::{IcrcAccountTransactionError, IcrcAccountTransactionRow};
use candid::Nat;
use std::{cmp::Ordering, str::FromStr};

pub(in crate::icrc) fn normalize_transaction_cursor(
    value: &str,
) -> Result<String, IcrcAccountTransactionError> {
    parse_transaction_cursor(value).map(|cursor| nat_text(&cursor))
}

pub(in crate::icrc) fn validate_canonical_account_transactions(
    transactions: &[IcrcAccountTransactionRow],
) -> Result<(), String> {
    let mut previous = None;
    for transaction in transactions {
        let normalized =
            normalize_transaction_cursor(&transaction.id).map_err(|error| error.to_string())?;
        if normalized != transaction.id {
            return Err(format!(
                "transaction id {} is not canonical decimal text",
                transaction.id
            ));
        }
        let current = Nat::from_str(&transaction.id)
            .map_err(|error| format!("invalid transaction id {}: {error}", transaction.id))?;
        if let Some(previous) = previous.as_ref()
            && current >= *previous
        {
            return Err("transactions are not unique newest-first rows".to_string());
        }
        previous = Some(current);
    }
    Ok(())
}

pub(super) fn parse_transaction_cursor(value: &str) -> Result<Nat, IcrcAccountTransactionError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(IcrcAccountTransactionError::InvalidCursor {
            value: value.to_string(),
            reason: "expected unsigned decimal text".to_string(),
        });
    }
    Nat::from_str(value).map_err(|error| IcrcAccountTransactionError::InvalidCursor {
        value: value.to_string(),
        reason: error.to_string(),
    })
}

pub(super) fn nat_text(value: &Nat) -> String {
    value.0.to_str_radix(10)
}

pub(super) fn compare_canonical_decimal(left: &str, right: &str) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nat_text_is_plain_decimal_for_reusable_pagination_cursors() {
        assert_eq!(nat_text(&Nat::from(779_513_u64)), "779513");
    }

    #[test]
    fn account_transaction_cursor_accepts_nat_beyond_u64_and_canonicalizes_zeroes() {
        assert_eq!(
            normalize_transaction_cursor("18446744073709551616")
                .expect("arbitrary candid Nat cursor"),
            "18446744073709551616"
        );
        assert_eq!(
            normalize_transaction_cursor("00042").expect("decimal cursor"),
            "42"
        );
    }
}
